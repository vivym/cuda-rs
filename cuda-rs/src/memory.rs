use crate::{ffi, stream::CuStream, error::{CuResult, CuError}};
use std::ffi::c_void;

pub struct HostMemory {
    ptr: *mut c_void,
    pub size: usize,
}

impl HostMemory {
    pub fn new(size: usize) -> CuResult<Self> {
        let mut ptr = std::ptr::null_mut();
        let res = unsafe {
            ffi::cuMemAllocHost_v2(&mut ptr, size)
        };

        wrap!(Self { ptr, size }, res)
    }

    pub unsafe fn from_raw(ptr: *mut c_void, size: usize) -> Self {
        Self { ptr, size }
    }

    pub unsafe fn get_raw(&self) -> *mut c_void {
        self.ptr
    }

    pub fn as_slice<T>(&self) -> &[T] {
        let size = self.size / std::mem::size_of::<T>();
        unsafe { std::slice::from_raw_parts(self.ptr as *const T, size) }
    }

    pub fn as_mut_slice<T>(&mut self) -> &mut [T] {
        let size = self.size / std::mem::size_of::<T>();
        unsafe { std::slice::from_raw_parts_mut(self.ptr as *mut T, size) }
    }

    pub fn copy_to_raw(&self, dst: *mut c_void, size: usize) -> CuResult<()> {
        let res = unsafe {
            ffi::cuMemcpy(dst as _, self.ptr as _, size)
        };

        wrap!((), res)
    }

    pub fn copy_from_raw(&self, src: *const c_void, size: usize) -> CuResult<()> {
        let res = unsafe {
            ffi::cuMemcpy(self.ptr as _, src as _, size)
        };

        wrap!((), res)
    }

    pub fn copy_to(&self, dst: &mut Self) -> CuResult<()> {
        self.copy_to_raw(dst.ptr, dst.size)
    }

    pub fn try_clone(&self) -> CuResult<Self> {
        let mut dst = Self::new(self.size)?;
        self.copy_to(&mut dst)?;

        Ok(dst)
    }
}

impl Drop for HostMemory {
    fn drop(&mut self) {
        unsafe { ffi::cuMemFreeHost(self.ptr) };
    }
}

pub struct DeviceMemory {
    ptr: ffi::CUdeviceptr,
    pub size: usize,
    pub stream: CuStream,
}

impl DeviceMemory {
    pub fn new(size: usize, stream: &CuStream) -> CuResult<Self> {
        let mut ptr: ffi::CUdeviceptr = 0;
        let res = unsafe {
            ffi::cuMemAllocAsync(
                &mut ptr, size, stream.get_raw()
            )
        };

        wrap!(Self { ptr, size, stream: stream.clone() }, res)
    }

    pub unsafe fn from_raw(ptr: ffi::CUdeviceptr, size: usize, stream: &CuStream) -> Self {
        Self { ptr, size, stream: stream.clone() }
    }

    pub unsafe fn get_raw(&self) -> ffi::CUdeviceptr {
        self.ptr
    }

    pub fn copy_to_raw(
        &self,
        dst: ffi::CUdeviceptr,
        size: usize,
        stream: Option<&CuStream>,
    ) -> CuResult<()> {
        let res = unsafe {
            let stream = stream
                .map_or(self.stream.get_raw(), |s| s.get_raw());
            ffi::cuMemcpyAsync(
                dst,
                self.ptr,
                size,
                stream,
            )
        };

        wrap!((), res)
    }

    pub fn copy_from_raw(
        &self,
        src: ffi::CUdeviceptr,
        size: usize,
        stream: Option<&CuStream>,
    ) -> CuResult<()> {
        let res = unsafe {
            let stream = stream
                .map_or(self.stream.get_raw(), |s| s.get_raw());
            ffi::cuMemcpyAsync(
                self.ptr,
                src,
                size,
                stream,
            )
        };

        wrap!((), res)
    }

    pub fn copy_to(&self, dst: &mut Self, stream: Option<&CuStream>) -> CuResult<()> {
        self.copy_to_raw(dst.ptr, dst.size, stream)
    }

    pub fn copy_from(&mut self, src: &Self, stream: Option<&CuStream>) -> CuResult<()> {
        src.copy_to(self, stream)
    }

    pub fn try_clone(&self) -> CuResult<Self> {
        let mut dst = Self::new(self.size, &self.stream)?;
        self.copy_to(&mut dst, None)?;

        Ok(dst)
    }

    pub fn to_host(&self) -> CuResult<HostMemory> {
        let host_mem = HostMemory::new(self.size)?;
        let res = unsafe {
            ffi::cuMemcpyAsync(
                host_mem.get_raw() as _,
                self.ptr,
                self.size,
                self.stream.get_raw(),
            )
        };

        wrap!(host_mem, res)
    }
}

impl Drop for DeviceMemory {
    fn drop(&mut self) {
        unsafe {
            ffi::cuMemFreeAsync(self.ptr, self.stream.get_raw())
        };
    }
}

const ALIGNMENT: usize = 512;

pub struct PitchedDeviceMemory {
    pub memory: DeviceMemory,
    pub pitch: usize,
    pub width: usize,
    pub height: usize,
}

impl PitchedDeviceMemory {
    pub fn new(width: usize, height: usize, stream: &CuStream) -> CuResult<Self> {
        let pitch = align_up(width, ALIGNMENT);

        let memory = DeviceMemory::new(pitch * height, stream)?;

        Ok(PitchedDeviceMemory {
            memory,
            pitch,
            width,
            height,
        })
    }

    pub unsafe fn get_raw(&self) -> ffi::CUdeviceptr {
        self.memory.get_raw()
    }

    pub fn copy_to_raw(
        &self,
        dst: ffi::CUdeviceptr,
        pitch: usize,
        width: usize,
        height: usize,
        is_dst_host: bool,
        stream: Option<&CuStream>,
    ) -> CuResult<()> {
        if width != self.width || height != self.height {
            return Err(CuError::InvalidValue);
        }

        let res = unsafe {
            let stream = stream
                .map_or(self.memory.stream.get_raw(), |s| s.get_raw());
            let mut params: ffi::CUDA_MEMCPY2D = std::mem::zeroed();
            params.srcMemoryType = ffi::CUmemorytype_enum_CU_MEMORYTYPE_DEVICE;
            params.srcDevice = self.memory.get_raw();
            params.srcPitch = self.pitch;
            params.dstMemoryType = if is_dst_host {
                ffi::CUmemorytype_enum_CU_MEMORYTYPE_HOST
            } else {
                ffi::CUmemorytype_enum_CU_MEMORYTYPE_DEVICE
            };
            params.dstDevice = dst;
            params.dstPitch = pitch;
            params.WidthInBytes = width;
            params.Height = height;
            ffi::cuMemcpy2DAsync_v2(&params, stream)
        };

        wrap!((), res)
    }

    pub fn copy_from_raw(
        &self,
        src: ffi::CUdeviceptr,
        pitch: usize,
        width: usize,
        height: usize,
        is_src_host: bool,
        stream: Option<&CuStream>,
    ) -> CuResult<()> {
        if width != self.width || height != self.height {
            return Err(CuError::InvalidValue);
        }

        let res = unsafe {
            let stream = stream
                .map_or(self.memory.stream.get_raw(), |s| s.get_raw());
            let mut params: ffi::CUDA_MEMCPY2D = std::mem::zeroed();
            params.srcMemoryType = if is_src_host {
                ffi::CUmemorytype_enum_CU_MEMORYTYPE_HOST
            } else {
                ffi::CUmemorytype_enum_CU_MEMORYTYPE_DEVICE
            };
            params.srcDevice = src;
            params.srcPitch = pitch;
            params.dstMemoryType = ffi::CUmemorytype_enum_CU_MEMORYTYPE_DEVICE;
            params.dstDevice = self.memory.get_raw();
            params.dstPitch = self.pitch;
            params.WidthInBytes = width;
            params.Height = height;
            ffi::cuMemcpy2DAsync_v2(&params, stream)
        };

        wrap!((), res)
    }

    pub fn copy_to(&self, dst: &mut Self, stream: Option<&CuStream>) -> CuResult<()> {
        self.copy_to_raw(
            dst.memory.ptr,
            dst.pitch,
            dst.width,
            dst.height,
            false,
            stream
        )
    }

    pub fn copy_from(&mut self, src: &Self, stream: Option<&CuStream>) -> CuResult<()> {
        src.copy_to(self, stream)
    }

    pub fn try_clone(&self) -> CuResult<Self> {
        let mut dst = Self::new(
            self.width, self.height, &self.memory.stream
        )?;
        self.copy_to(&mut dst, None)?;

        Ok(dst)
    }

    pub fn to_host(&self) -> CuResult<HostMemory> {
        let host_size = self.width * self.height;
        let host_mem = HostMemory::new(host_size)?;
        unsafe {
            self.copy_to_raw(
                host_mem.get_raw() as _,
                self.width,
                self.width,
                self.height,
                true,
                None,
            )
        }?;

        Ok(host_mem)
    }

    pub fn stream(&self) -> &CuStream {
        &self.memory.stream
    }
}

#[inline]
fn align_up(x: usize, align: usize) -> usize {
    assert!(align.is_power_of_two(), "`align` must be a power of two");
    let align_mask = align - 1;
    if x & align_mask == 0 {
        x   // already aligned
    } else {
        if let Some(aligned) = (x | align_mask).checked_add(1) {
            aligned
        } else {
            panic!("attempt to add with overflow")
        }
    }
}

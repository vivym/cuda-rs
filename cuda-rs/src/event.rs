use crate::{
    error::{CuError, CuResult},
    ffi,
    stream::CuStream,
};

pub struct CuEvent(ffi::CUevent);

impl CuEvent {
    pub fn new() -> CuResult<Self> {
        let mut s = CuEvent(std::ptr::null_mut());
        let res = unsafe {
            ffi::cuEventCreate(&mut s.0, ffi::CUevent_flags_enum_CU_EVENT_DEFAULT)
        };

        wrap!(s, res)
    }

    pub fn record(&self, stream: &CuStream) -> CuResult<()> {
        let res = unsafe {
            ffi::cuEventRecord(self.0, stream.get_raw())
        };

        wrap!((), res)
    }

    pub fn query(&self) -> CuResult<bool> {
        let res = unsafe { ffi::cuEventQuery(self.0) };

        if res == ffi::cudaError_enum_CUDA_SUCCESS || res == ffi::cudaError_enum_CUDA_ERROR_NOT_READY {
            Ok(res == ffi::cudaError_enum_CUDA_SUCCESS)
        } else {
            wrap!(false, res)
        }
    }

    pub fn synchronize(&self) -> CuResult<()> {
        let res = unsafe { ffi::cuEventSynchronize(self.0) };

        wrap!((), res)
    }

    pub fn elapsed_time(&self, start: &CuEvent) -> CuResult<f32> {
        let mut ms = 0.0;
        let res = unsafe {
            ffi::cuEventElapsedTime(&mut ms as *mut f32, start.0, self.0)
        };

        wrap!(ms, res)
    }

    pub unsafe fn get_raw(&self) -> ffi::CUevent {
        self.0
    }
}

impl Drop for CuEvent {
    fn drop(&mut self) {
        unsafe { ffi::cuEventDestroy_v2(self.0) };
    }
}

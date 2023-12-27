use crate::{
    error::{CuError, CuResult},
    ffi::{
        CUdevice,
        cuDeviceGet,
        cuDeviceGetCount,
        cuDeviceTotalMem_v2,
    },
};
use num_traits::FromPrimitive;
use std::os::raw::c_int;

pub struct CuDevice(pub CUdevice);

impl CuDevice {
    pub fn new(ordinal: u32) -> CuResult<Self> {
        let mut d = CuDevice(0);
        let res = unsafe { cuDeviceGet(&mut d.0 as *mut i32, ordinal as c_int) };

        wrap!(d, res)
    }

    pub fn get_device_count() -> CuResult<i32> {
        let mut count = 0;
        let res = unsafe { cuDeviceGetCount(&mut count as *mut i32) };

        wrap!(count, res)
    }

    pub fn total_memory(&self) -> CuResult<usize> {
        let mut nbytes = 0;
        let res = unsafe { cuDeviceTotalMem_v2(&mut nbytes as *mut usize, self.0) };

        wrap!(nbytes, res)
    }
}

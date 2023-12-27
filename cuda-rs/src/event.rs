use crate::{
    error::{CuError, CuResult},
    ffi::{
        CUevent,
        cuEventCreate,
        cuEventDestroy_v2,
        cuEventRecord,
        cuEventQuery,
        cuEventSynchronize,
        cuEventElapsedTime,
        CUevent_flags_enum_CU_EVENT_DEFAULT,
    },
    stream::CuStream,
};
use num_traits::FromPrimitive;

pub struct CuEvent(CUevent);

impl CuEvent {
    pub fn new() -> CuResult<Self> {
        let mut s = CuEvent(std::ptr::null_mut());
        let res = unsafe {
            cuEventCreate(&mut s.0, CUevent_flags_enum_CU_EVENT_DEFAULT)
        };

        wrap!(s, res)
    }

    pub fn record(&self, stream: &CuStream) -> CuResult<()> {
        let res = unsafe { cuEventRecord(self.0, stream.0) };

        wrap!((), res)
    }

    pub fn query(&self) -> CuResult<bool> {
        let res = unsafe { cuEventQuery(self.0) };

        wrap!(res == 0, res)
    }

    pub fn synchronize(&self) -> CuResult<()> {
        let res = unsafe { cuEventSynchronize(self.0) };

        wrap!((), res)
    }

    pub fn elapsed_time(&self, start: &CuEvent) -> CuResult<f32> {
        let mut ms = 0.0;
        let res = unsafe {
            cuEventElapsedTime(&mut ms as *mut f32, start.0, self.0)
        };

        wrap!(ms, res)
    }
}

impl Drop for CuEvent {
    fn drop(&mut self) {
        unsafe { cuEventDestroy_v2(self.0) };
    }
}

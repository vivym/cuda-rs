use crate::{
    error::{CuError, CuResult},
    ffi::{
        CUstream,
        cuStreamCreate,
        cuStreamDestroy_v2,
        cuStreamSynchronize,
        CUstream_flags_enum_CU_STREAM_DEFAULT,
    },
};
use num_traits::FromPrimitive;

pub struct CuStream(pub CUstream);

impl CuStream {
    pub fn new() -> CuResult<Self> {
        let mut s = CuStream(std::ptr::null_mut());
        let res = unsafe {
            cuStreamCreate(&mut s.0, CUstream_flags_enum_CU_STREAM_DEFAULT)
        };

        wrap!(s, res)
    }

    pub fn synchronize(&self) -> CuResult<()> {
        let res = unsafe { cuStreamSynchronize(self.0) };

        wrap!((), res)
    }
}

impl Drop for CuStream {
    fn drop(&mut self) {
        unsafe { cuStreamDestroy_v2(self.0) };
    }
}

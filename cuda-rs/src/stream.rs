use crate::{
    context::CuContext,
    error::CuResult,
    event::CuEvent,
    ffi,
};
use std::sync::Arc;

struct CUstream(ffi::CUstream);

impl Drop for CUstream {
    fn drop(&mut self) {
        unsafe { ffi::cuStreamDestroy_v2(self.0) };
    }
}

enum Inner {
    Owned(Arc<CUstream>),
    Borrowed(ffi::CUstream),
}

pub struct CuStream(Inner);

impl CuStream {
    pub fn new() -> CuResult<Self> {
        let mut s = std::ptr::null_mut();
        let res = unsafe {
            ffi::cuStreamCreate(&mut s, ffi::CUstream_flags_enum_CU_STREAM_DEFAULT)
        };
        let s = CuStream(Inner::Owned(Arc::new(CUstream(s))));

        wrap!(s, res)
    }

    pub unsafe fn from_raw(s: ffi::CUstream) -> Self {
        CuStream(Inner::Borrowed(s))
    }

    pub fn synchronize(&self) -> CuResult<()> {
        let res = unsafe { ffi::cuStreamSynchronize(self.get_raw()) };

        wrap!((), res)
    }

    pub fn get_context(&self) -> CuResult<CuContext> {
        let mut ctx = std::ptr::null_mut();

        let (ctx, res) = unsafe {
            let raw_stream = self.get_raw();
            let res = ffi::cuStreamGetCtx(raw_stream, &mut ctx);
            let ctx = CuContext::from_raw(ctx);

            (ctx, res)
        };

        wrap!(ctx, res)
    }

    pub fn query(&self) -> CuResult<bool> {
        let res = unsafe { ffi::cuStreamQuery(self.get_raw()) };

        if res == ffi::cudaError_enum_CUDA_SUCCESS || res == ffi::cudaError_enum_CUDA_ERROR_NOT_READY {
            Ok(res == ffi::cudaError_enum_CUDA_SUCCESS)
        } else {
            wrap!(false, res)
        }
    }

    pub fn wait_on_event(&self, event: &CuEvent) -> CuResult<()> {
        let res = unsafe {
            ffi::cuStreamWaitEvent(self.get_raw(), event.get_raw(), 0)
        };

        wrap!((), res)
    }

    pub unsafe fn get_raw(&self) -> ffi::CUstream {
        match self.0 {
            Inner::Owned(ref s) => s.0,
            Inner::Borrowed(s) => s,
        }
    }
}

impl Clone for CuStream {
    fn clone(&self) -> Self {
        match self.0 {
            Inner::Owned(ref s) => CuStream(Inner::Owned(s.clone())),
            Inner::Borrowed(s) => CuStream(Inner::Borrowed(s)),
        }
    }
}

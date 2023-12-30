use crate::{
    device::CuDevice,
    error::CuResult,
    ffi,
};
use std::sync::Arc;

struct CUcontext(ffi::CUcontext);

impl Drop for CUcontext {
    fn drop(&mut self) {
        unsafe { ffi::cuCtxDestroy_v2(self.0) };
    }
}

enum Inner {
    Owned(Arc<CUcontext>),
    Borrowed(ffi::CUcontext),
}

pub struct CuContext(Inner);

impl CuContext {
    pub fn new(device: &CuDevice) -> CuResult<Self> {
        let mut ctx = std::ptr::null_mut();
        let res = unsafe {
            ffi::cuCtxCreate_v2(&mut ctx, 0, device.get_raw())
        };
        let ctx = CuContext(Inner::Owned(Arc::new(CUcontext(ctx))));

        wrap!(ctx, res)
    }

    pub unsafe fn from_raw(ctx: ffi::CUcontext) -> Self {
        CuContext(Inner::Borrowed(ctx))
    }

    pub fn retain_primary_context(device: &CuDevice) -> CuResult<Self> {
        let mut ctx = std::ptr::null_mut();
        let res = unsafe {
            ffi::cuDevicePrimaryCtxRetain(&mut ctx, device.get_raw())
        };
        let ctx = CuContext(Inner::Borrowed(ctx));

        wrap!(ctx, res)
    }

    pub fn current() -> CuResult<Self> {
        let mut ctx = std::ptr::null_mut();
        let res = unsafe { ffi::cuCtxGetCurrent(&mut ctx) };
        let ctx = CuContext(Inner::Borrowed(ctx));

        wrap!(ctx, res)
    }

    pub unsafe fn get_raw(&self) -> ffi::CUcontext {
        match self.0 {
            Inner::Owned(ref ctx) => ctx.0,
            Inner::Borrowed(ctx) => ctx,
        }
    }

    pub fn push(&self) -> CuResult<()> {
        let res = unsafe {
            let ctx = self.get_raw();
            ffi::cuCtxPushCurrent_v2(ctx)
        };

        wrap!((), res)
    }

    pub fn pop() -> CuResult<()> {
        let res = unsafe {
            ffi::cuCtxPopCurrent_v2(&mut std::ptr::null_mut())
        };

        wrap!((), res)
    }

    pub fn guard(self) -> CuResult<CuContextGuard> {
        CuContextGuard::new(self)
    }
}

impl Clone for CuContext {
    fn clone(&self) -> Self {
        match self.0 {
            Inner::Owned(ref ctx) => CuContext(Inner::Owned(ctx.clone())),
            Inner::Borrowed(ctx) => CuContext(Inner::Borrowed(ctx)),
        }
    }
}

pub struct CuContextGuard(pub CuContext);

impl CuContextGuard {
    pub fn new(ctx: CuContext) -> CuResult<Self> {
        ctx.push()?;
        Ok(CuContextGuard(ctx))
    }
}

impl Drop for CuContextGuard {
    fn drop(&mut self) {
        CuContext::pop().unwrap();
    }
}

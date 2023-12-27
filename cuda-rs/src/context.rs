use crate::{
    ffi::cuda::{
        CUcontext,
        cuCtxCreate_v2,
        cuCtxDestroy_v2,
        cuDevicePrimaryCtxRetain,
        cuCtxPushCurrent_v2,
        cuCtxPopCurrent_v2,
        cuCtxGetCurrent,
    },
    device::CuDevice,
    error::{CuError, CuResult},
};
use num_traits::FromPrimitive;

pub struct CuContext {
    pub ctx: CUcontext,
    is_owner: bool,
}

impl CuContext {
    pub fn new(device: &CuDevice) -> CuResult<Self> {
        let mut ctx = CuContext{
            ctx: std::ptr::null_mut(),
            is_owner: true,
        };
        let res = unsafe { cuCtxCreate_v2(&mut ctx.ctx, 0, device.0) };

        wrap!(ctx, res)
    }

    pub fn retain_primary_context(device: &CuDevice) -> CuResult<Self> {
        let mut ctx = CuContext{
            ctx: std::ptr::null_mut(),
            is_owner: false,
        };
        let res = unsafe { cuDevicePrimaryCtxRetain(&mut ctx.ctx, device.0) };

        wrap!(ctx, res)
    }

    pub fn current() -> CuResult<Self> {
        let mut ctx = CuContext{
            ctx: std::ptr::null_mut(),
            is_owner: false,
        };
        let res = unsafe { cuCtxGetCurrent(&mut ctx.ctx) };

        wrap!(ctx, res)
    }

    pub fn push(&self) -> CuResult<()> {
        let res = unsafe { cuCtxPushCurrent_v2(self.ctx) };

        wrap!((), res)
    }

    pub fn pop(&self) -> CuResult<()> {
        let res = unsafe { cuCtxPopCurrent_v2(&mut std::ptr::null_mut()) };

        wrap!((), res)
    }

    pub fn guard(self) -> CuResult<CuContextGuard> {
        CuContextGuard::new(self)
    }
}

impl Drop for CuContext {
    fn drop(&mut self) {
        if self.is_owner {
            unsafe { cuCtxDestroy_v2(self.ctx) };
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
        self.0.pop().unwrap();
    }
}

#[macro_use]
extern crate enum_primitive;

extern crate cuda_rs_sys as ffi;

#[macro_use]
mod macros;

pub mod context;
pub mod device;
pub mod error;
pub mod event;
pub mod stream;

pub fn init() -> Result<(), error::CuError> {
    let res = unsafe { ffi::cuInit(0) };
    wrap!((), res)
}

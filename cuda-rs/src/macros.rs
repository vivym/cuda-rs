macro_rules! wrap {
    ($val:expr, $res:ident) => (
        if $res == crate::ffi::cudaError_enum_CUDA_SUCCESS {
            Ok($val)
        } else {
            use num_traits::FromPrimitive;
            Err(CuError::from_u32($res).unwrap())
        }
    )
}

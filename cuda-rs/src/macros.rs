macro_rules! wrap {
    ($val:expr, $res:ident) => (
        if $res == crate::ffi::cuda::cudaError_enum_CUDA_SUCCESS {
            Ok($val)
        } else {
            Err(CuError::from_u32($res).unwrap())
        }
    )
}

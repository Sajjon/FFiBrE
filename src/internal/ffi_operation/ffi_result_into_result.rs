use crate::prelude::*;

pub trait FFIResult<T>: Into<Result<T, SwiftSideError>> {}
use crate::prelude::*;

pub trait IsResultListener: From<FFIOperationResultListener<Self::OpResult>> {
    type Request;
    type OpResult: Into<Result<Self::Response, FFISideError>>;
    type Response;
}

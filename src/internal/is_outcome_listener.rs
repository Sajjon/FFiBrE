use crate::prelude::*;

pub trait IsOutcomeListener: From<FFIOperationOutcomeListener<Self::Outcome>> {
    type Request;
    type Response;
    type Failure: Into<FFISideError>;
    type Outcome: Into<Result<Self::Response, Self::Failure>>;
}

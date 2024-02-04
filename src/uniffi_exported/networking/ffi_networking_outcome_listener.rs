use crate::prelude::*;

#[derive(Object)]
pub struct FFINetworkingOutcomeListener {
    result_listener: FFIOperationOutcomeListener<FFINetworkingOutcome>,
}
impl IsOutcomeListener for FFINetworkingOutcomeListener {
    type Request = FFINetworkingRequest;
    type Response = FFINetworkingResponse;
    type Failure = FFINetworkingError;
    type Outcome = FFINetworkingOutcome;
}

impl From<FFIOperationOutcomeListener<FFINetworkingOutcome>> for FFINetworkingOutcomeListener {
    fn from(value: FFIOperationOutcomeListener<FFINetworkingOutcome>) -> Self {
        Self::with_result_listener(value)
    }
}
impl FFINetworkingOutcomeListener {
    pub fn with_result_listener(
        result_listener: FFIOperationOutcomeListener<FFINetworkingOutcome>,
    ) -> Self {
        Self { result_listener }
    }
}

#[export]
impl FFINetworkingOutcomeListener {
    fn notify_outcome(&self, result: FFINetworkingOutcome) {
        self.result_listener.notify_outcome(result.into())
    }
}

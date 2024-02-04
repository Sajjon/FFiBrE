use crate::prelude::*;

#[derive(Object)]
pub struct FFIFileIOWriteOutcomeListener {
    result_listener: FFIOperationOutcomeListener<FFIFileIOWriteOutcome>,
}
impl IsOutcomeListener for FFIFileIOWriteOutcomeListener {
    type Request = FFIFileIOWriteRequest;
    type Response = FFIFileIOWriteResponse;
    type Failure = FFIFileIOWriteError;
    type Outcome = FFIFileIOWriteOutcome;
}
impl From<FFIOperationOutcomeListener<FFIFileIOWriteOutcome>> for FFIFileIOWriteOutcomeListener {
    fn from(value: FFIOperationOutcomeListener<FFIFileIOWriteOutcome>) -> Self {
        Self::with_result_listener(value)
    }
}
impl FFIFileIOWriteOutcomeListener {
    pub fn with_result_listener(
        result_listener: FFIOperationOutcomeListener<FFIFileIOWriteOutcome>,
    ) -> Self {
        Self { result_listener }
    }
}

#[export]
impl FFIFileIOWriteOutcomeListener {
    /// This is called from FFI Side (Swift side), inside the implementation of
    /// an `execute_file_io_write:request:listener_rust_side` method on a [`FFIOperationExecutor`],
    /// when the operation has finished, with the [`FFIFileIOWriteOutcome`].
    fn notify_outcome(&self, result: FFIFileIOWriteOutcome) {
        self.result_listener.notify_outcome(result.into())
    }
}

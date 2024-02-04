use crate::prelude::*;

#[derive(Object)]
pub struct FFIFileIOReadOutcomeListener {
    result_listener: FFIOperationOutcomeListener<FFIFileIOReadOutcome>,
}
impl IsOutcomeListener for FFIFileIOReadOutcomeListener {
    type Request = FFIFileIOReadRequest;
    type Response = FFIFileIOReadResponse;
    type Failure = FFIFileIOReadError;
    type Outcome = FFIFileIOReadOutcome;
}
impl From<FFIOperationOutcomeListener<FFIFileIOReadOutcome>> for FFIFileIOReadOutcomeListener {
    fn from(value: FFIOperationOutcomeListener<FFIFileIOReadOutcome>) -> Self {
        Self::with_result_listener(value)
    }
}
impl FFIFileIOReadOutcomeListener {
    pub fn with_result_listener(
        result_listener: FFIOperationOutcomeListener<FFIFileIOReadOutcome>,
    ) -> Self {
        Self { result_listener }
    }
}

#[export]
impl FFIFileIOReadOutcomeListener {
    /// This is called from FFI Side (Swift side), inside the implementation of
    /// an `execute_file_io_read:request:listener_rust_side` method on a [`FFIOperationExecutor`],
    /// when the operation has finished, with the [`FFIFileIOReadOutcome`].
    fn notify_outcome(&self, result: FFIFileIOReadOutcome) {
        self.result_listener.notify_outcome(result.into())
    }
}

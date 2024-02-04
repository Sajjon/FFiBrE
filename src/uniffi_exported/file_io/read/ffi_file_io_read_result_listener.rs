use crate::prelude::*;

#[derive(Object)]
pub struct FFIFileIOReadResultListener {
    result_listener: FFIOperationResultListener<FFIFileIOReadResult>,
}
impl IsResultListener for FFIFileIOReadResultListener {
    type OpResult = FFIFileIOReadResult;
    type Request = FFIFileIOReadRequest;
    type Response = FFIFileIOReadResponse;
}
impl From<FFIOperationResultListener<FFIFileIOReadResult>> for FFIFileIOReadResultListener {
    fn from(value: FFIOperationResultListener<FFIFileIOReadResult>) -> Self {
        Self::with_result_listener(value)
    }
}
impl FFIFileIOReadResultListener {
    pub fn with_result_listener(
        result_listener: FFIOperationResultListener<FFIFileIOReadResult>,
    ) -> Self {
        Self { result_listener }
    }
}

#[export]
impl FFIFileIOReadResultListener {
    /// This is called from FFI Side (Swift side), inside the implementation of
    /// an `execute_file_io_read:request:listener_rust_side` method on a [`FFIOperationHandler`],
    /// when the operation has finished, with the [`FFIFileIOReadResult`].
    fn notify_result(&self, result: FFIFileIOReadResult) {
        self.result_listener.notify_result(result.into())
    }
}

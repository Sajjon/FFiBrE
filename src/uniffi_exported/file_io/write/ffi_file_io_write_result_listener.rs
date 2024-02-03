use crate::prelude::*;

#[derive(Object)]
pub struct FFIFileIOWriteResultListener {
    result_listener: FFIOperationResultListener,
}
impl ResultListener for FFIFileIOWriteResultListener {}
impl From<FFIOperationResultListener> for FFIFileIOWriteResultListener {
    fn from(value: FFIOperationResultListener) -> Self {
        Self::with_result_listener(value)
    }
}
impl FFIFileIOWriteResultListener {
    pub fn with_result_listener(result_listener: FFIOperationResultListener) -> Self {
        Self { result_listener }
    }
}

#[export]
impl FFIFileIOWriteResultListener {
    /// This is called from FFI Side (Swift side), inside the implementation of
    /// an `execute_file_io_write:request:listener_rust_side` method on a [`FFIOperationHandler`],
    /// when the operation has finished, with the [`FFIFileIOWriteResult`].
    fn notify_result(&self, result: FFIFileIOWriteResult) {
        self.result_listener.notify_result(result.into())
    }
}

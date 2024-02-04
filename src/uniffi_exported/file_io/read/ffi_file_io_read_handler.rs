use crate::prelude::*;

#[uniffi::export(with_foreign)]
pub trait FFIFileIOReadHandler: FFIOperationHandler<FFIFileIOReadResultListener> {
    /// Rust will tell the handler to execute `operation` by calling this
    /// function, which a concrete type FFI side (Swift side) has implemented.
    /// Once the operation has finished with a result (Success/Failure) it
    /// passes back the result using the `listener_rust_side` callback.
    fn execute_file_io_read_request(
        &self,
        request: FFIFileIOReadRequest,
        listener_rust_side: Arc<FFIFileIOReadResultListener>,
    ) -> Result<(), FFISideError>;
}

impl<U: FFIFileIOReadHandler> FFIOperationHandler<FFIFileIOReadResultListener> for U {
    fn execute_operation(
        &self,
        operation: <FFIFileIOReadResultListener as IsResultListener>::Request,
        listener_rust_side: FFIFileIOReadResultListener,
    ) -> Result<(), FFISideError> {
        self.execute_file_io_read_request(operation, listener_rust_side.into())
    }
}

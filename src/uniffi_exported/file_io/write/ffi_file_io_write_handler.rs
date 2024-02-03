use crate::prelude::*;

#[uniffi::export(with_foreign)]
pub trait FFIFileIOWriteHandler: FFIOperationHandler<FFIFileIOWriteResultListener> {
    /// Rust will tell the handler to execute `operation` by calling this
    /// function, which a concrete type FFI side (Swift side) has implemented.
    /// Once the operation has finished with a result (Success/Failure) it
    /// passes back the result using the `listener_rust_side` callback.
    fn execute_file_io_write_request(
        &self,
        request: FFIFileIOWriteRequest,
        listener_rust_side: Arc<FFIFileIOWriteResultListener>,
    ) -> Result<(), SwiftSideError>;
}



impl<U: FFIFileIOWriteHandler> FFIOperationHandler<FFIFileIOWriteResultListener> for U {
    fn execute_operation(
        &self,
        operation: <FFIFileIOWriteResultListener as ResultListener>::Request,
        listener_rust_side: FFIFileIOWriteResultListener,
    ) -> Result<(), SwiftSideError> {
        self.execute_file_io_write_request(operation, listener_rust_side.into())
    }
}

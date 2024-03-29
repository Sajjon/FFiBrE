use crate::prelude::*;

#[uniffi::export(with_foreign)]
pub trait FFIFileIOReadExecutor: FFIOperationExecutor<FFIFileIOReadOutcomeListener> {
    /// Rust will tell the handler to execute `operation` by calling this
    /// function, which a concrete type FFI side (Swift side) has implemented.
    /// Once the operation has finished with a result (Success/Failure) it
    /// passes back the result using the `listener_rust_side` callback.
    fn execute_file_io_read_request(
        &self,
        request: FFIFileIOReadRequest,
        listener_rust_side: Arc<FFIFileIOReadOutcomeListener>,
    ) -> Result<(), FFISideError>;
}

impl<U: FFIFileIOReadExecutor> FFIOperationExecutor<FFIFileIOReadOutcomeListener> for U {
    fn execute_request(
        &self,
        request: <FFIFileIOReadOutcomeListener as IsOutcomeListener>::Request,
        listener_rust_side: FFIFileIOReadOutcomeListener,
    ) -> Result<(), FFISideError> {
        self.execute_file_io_read_request(request, listener_rust_side.into())
    }
}

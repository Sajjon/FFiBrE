use crate::prelude::*;

#[uniffi::export(with_foreign)]
pub trait FFINetworkingExecutor: FFIOperationExecutor<FFINetworkingOutcomeListener> {
    /// Rust will tell the handler to execute `operation` by calling this
    /// function, which a concrete type FFI side (Swift side) has implemented.
    /// Once the operation has finished with a result (Success/Failure) it
    /// passes back the result using the `listener_rust_side` callback.
    fn execute_network_request(
        &self,
        request: NetworkRequest,
        listener_rust_side: Arc<FFINetworkingOutcomeListener>,
    ) -> Result<(), FFISideError>;
}

impl<U: FFINetworkingExecutor> FFIOperationExecutor<FFINetworkingOutcomeListener> for U {
    fn execute_request(
        &self,
        request: <FFINetworkingOutcomeListener as IsOutcomeListener>::Request,
        listener_rust_side: FFINetworkingOutcomeListener,
    ) -> Result<(), FFISideError> {
        self.execute_network_request(request, listener_rust_side.into())
    }
}

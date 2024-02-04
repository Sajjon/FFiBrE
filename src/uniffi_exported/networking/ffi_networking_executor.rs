use crate::prelude::*;

#[uniffi::export(with_foreign)]
pub trait FFINetworkingExecutor: FFIOperationExecutor<FFINetworkingOutcomeListener> {
    fn execute_networking_request(
        &self,
        request: FFINetworkingRequest,
        listener_rust_side: Arc<FFINetworkingOutcomeListener>,
    ) -> Result<(), FFISideError>;
}

impl<U: FFINetworkingExecutor> FFIOperationExecutor<FFINetworkingOutcomeListener> for U {
    fn execute_request(
        &self,
        request: <FFINetworkingOutcomeListener as IsOutcomeListener>::Request,
        listener_rust_side: FFINetworkingOutcomeListener,
    ) -> Result<(), FFISideError> {
        self.execute_networking_request(request, listener_rust_side.into())
    }
}

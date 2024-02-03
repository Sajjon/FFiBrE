use crate::prelude::*;

pub struct FFINetworkRequestDispatcher {
    pub dispatcher: FFIOperationDispatcher<FFINetworkingResultListener>,
}

impl FFINetworkRequestDispatcher {
    pub fn with_dispatcher(
        dispatcher: FFIOperationDispatcher<FFINetworkingResultListener>,
    ) -> Self {
        Self { dispatcher }
    }
    pub fn new(network_antenna: Arc<dyn FFINetworkingHandler>) -> Self {
        Self::with_dispatcher(FFIOperationDispatcher::new(network_antenna))
    }
    pub(crate) async fn dispatch_network_request(
        &self,
        request: NetworkRequest,
    ) -> Result<NetworkResponse, NetworkError> {
        self.dispatcher
            .dispatch(FFIOperation::Networking { request })
            .await
            .and_then(|r| {
                r.into_networking().map_err(|_| NetworkError::FromRust {
                    error: RustSideError::WrongFFIOperationOKExpectedNetworkResponse,
                })
            })
    }
}

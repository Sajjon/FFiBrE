use crate::prelude::*;

impl FFINetworkRequestDispatcher {
    pub(crate) async fn dispatch_network_request(
        &self,
        request: NetworkRequest,
    ) -> Result<NetworkResponse, NetworkError> {
        self.dispatcher.dispatch(request).await
    }
}

impl<L: ResultListener> FFIOperationDispatcher<L> {
    pub(crate) async fn dispatch(
        &self,
        operation: L::Request,
    ) -> Result<L::Response, NetworkError> {
        // Underlying tokio channel used to get result from Swift back to Rust.
        let (sender, receiver) = channel::<L::OpResult>();

        // Our callback we pass to Swift
        let result_listener = FFIOperationResultListener::new(sender);

        // Make request
        self.handler
            .execute_operation(
                // Pass operation to Swift to make
                operation,
                // Pass callback, Swift will call `result_listener.notify_result`
                result_listener.into(),
            )
            .map_err(|e| NetworkError::from(e))?;

        // Await response from Swift
        let response = receiver.await.map_err(|_| NetworkError::FromRust {
            error: RustSideError::FailedToReceiveResponseFromSwift,
        })?;

        // Result::<L::Response, SwiftSideError>::from(response).map_err(|e| e.into())
        response.into().map_err(|e| e.into())
    }
}

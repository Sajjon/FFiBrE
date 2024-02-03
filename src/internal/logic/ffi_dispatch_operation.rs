use crate::prelude::*;

impl<L: IsResultListener> FFIOperationDispatcher<L> {
    pub(crate) async fn dispatch(
        &self,
        operation: L::Request,
    ) -> Result<L::Response, FFIBridgeError> {
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
            .map_err(|e| FFIBridgeError::from(e))?;

        // Await response from Swift
        let response = receiver.await.map_err(|_| FFIBridgeError::FromRust {
            error: RustSideError::FailedToReceiveResponseFromSwift,
        })?;

        // Result::<L::Response, FFISideError>::from(response).map_err(|e| e.into())
        response.into().map_err(|e| e.into())
    }
}

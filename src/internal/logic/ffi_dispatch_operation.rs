use crate::prelude::*;

impl<L: ResultListener> FFIOperationDispatcher<L> {
    pub(crate) async fn dispatch(
        &self,
        operation: FFIOperation,
    ) -> Result<FFIOperationOk, NetworkError> {
        if !self
            .handler
            .supported_operations()
            .contains(&operation.operation_kind())
        {
            panic!("Unsupported operation: {:?}", operation)
        }

        // Underlying tokio channel used to get result from Swift back to Rust.
        let (sender, receiver) = channel::<FFIOperationResult>();

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

        // Map response from Swift -> Result<Option<Vec<u8>>, NetworkError>,
        // keeping any errors happening in Swift intact.
        Result::<FFIOperationOk, SwiftSideError>::from(response).map_err(|e| e.into())
    }
}

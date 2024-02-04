use crate::prelude::*;

pub struct FFIOperationDispatcher<L: IsOutcomeListener> {
    pub executor: Arc<dyn FFIOperationExecutor<L>>,
}

impl<L: IsOutcomeListener> FFIOperationDispatcher<L> {
    pub fn new(handler: Arc<dyn FFIOperationExecutor<L>>) -> Self {
        Self { executor: handler }
    }

    pub(crate) async fn dispatch(
        &self,
        operation: L::Request,
    ) -> Result<L::Response, FFIBridgeError> {
        // Underlying tokio channel used to get result from Swift back to Rust.
        let (sender, receiver) = channel::<L::Outcome>();

        // Our callback we pass to Swift
        let outcome_listener = FFIOperationOutcomeListener::new(sender);

        // Make request
        self.executor
            .execute_request(
                // Pass operation to Swift to make
                operation,
                // Pass callback, Swift will call `result_listener.notify_outcome`
                outcome_listener.into(),
            )
            .map_err(|e| FFIBridgeError::from(e))?;

        // Await response from Swift
        let response = receiver.await.map_err(|_| FFIBridgeError::FromRust {
            error: RustSideError::FailedToReceiveResponseFromSwift,
        })?;

        response.into().map_err(|e| e.into().into())
    }
}

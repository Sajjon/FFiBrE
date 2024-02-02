use crate::prelude::*;

impl HTTPClient {
    pub(crate) async fn make_request(
        &self,
        request: NetworkRequest,
    ) -> Result<NetworkResponse, NetworkError> {
        // Underlying tokio channel used to get result from Swift back to Rust.
        let (sender, receiver) = channel::<NetworkResult>();

        // Our callback we pass to Swift
        let network_result_listener = NetworkResultListener::new(sender);

        // Make request
        self.network_antenna
            .make_request(
                // Pass request to Swift to make
                request,
                // Pass callback, Swift will call `network_result_listener.notify_result`
                network_result_listener.into(),
            )
            .map_err(|e| NetworkError::from(e))?;

        // Await response from Swift
        let response = receiver.await.map_err(|_| NetworkError::FromRust {
            error: RustSideError::FailedToReceiveResponseFromSwift,
        })?;

        // Map response from Swift (which is a NetworkResponse) -> Result<NetworkResponse, NetworkError>,
        // keeping any errors happening in Swift intact.
        Result::<NetworkResponse, SwiftSideError>::from(response).map_err(|e| e.into())
    }
}

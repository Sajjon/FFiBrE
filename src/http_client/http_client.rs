use crate::prelude::*;
use tokio::sync::oneshot;

#[derive(Object)]
pub struct HTTPClient {
    request_sender: Arc<dyn HTTPClientRequestSender>,
}

#[export]
impl HTTPClient {
    #[uniffi::constructor]
    pub fn new(request_sender: Arc<dyn HTTPClientRequestSender>) -> Self {
        Self { request_sender }
    }
}

impl HTTPClient {
    pub(crate) async fn make_request(
        &self,
        request: NetworkRequest,
    ) -> Result<NetworkResponse, NetworkError> {
        let (response_sender, response_receiver) = oneshot::channel();
        let sender_wrapper = OneshotSenderWrapper::new(response_sender);
        self.request_sender
            .send(request, Arc::new(sender_wrapper))
            .unwrap();

        response_receiver
            .await
            .map_err(|_| NetworkError::FailedToReceiveResponseFromSwift)
            .and_then(|r| r.into())
    }
}

#[derive(Object)]
pub struct OneshotSenderWrapper(Mutex<Option<oneshot::Sender<NetworkResult>>>);
impl OneshotSenderWrapper {
    pub fn new(sender: oneshot::Sender<NetworkResult>) -> Self {
        Self(Mutex::new(Some(sender)))
    }
}
unsafe impl Send for OneshotSenderWrapper {}
unsafe impl Sync for OneshotSenderWrapper {}

impl NotifyRustFromSwift for OneshotSenderWrapper {
    fn response(&self, result: NetworkResult) {
        let sender = self.0.lock().unwrap().take().unwrap();
        sender.send(result).unwrap();
    }
}

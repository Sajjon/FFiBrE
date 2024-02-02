use crate::prelude::*;

#[derive(Object)]
pub struct HTTPClient {
    pub request_sender: Arc<dyn HTTPClientRequestSender>,
}

#[export]
impl HTTPClient {
    #[uniffi::constructor]
    pub fn new(request_sender: Arc<dyn HTTPClientRequestSender>) -> Self {
        Self { request_sender }
    }
}

pub struct OneshotSenderWrapper(Mutex<Option<Sender<NetworkResult>>>);

impl OneshotSenderWrapper {
    pub fn new(sender: Sender<NetworkResult>) -> Self {
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

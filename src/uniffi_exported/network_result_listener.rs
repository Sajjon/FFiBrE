use crate::prelude::*;

#[derive(Object)]
pub struct NetworkResultListener {
    sender: Mutex<Option<Sender<NetworkResult>>>,
}

impl NetworkResultListener {
    pub fn new(sender: Sender<NetworkResult>) -> Self {
        Self {
            sender: Mutex::new(Some(sender)),
        }
    }
}
unsafe impl Send for NetworkResultListener {}
unsafe impl Sync for NetworkResultListener {}

#[export]
impl NetworkResultListener {
    fn notify_result(&self, result: NetworkResult) {
        let sender = self.sender.lock().unwrap().take().unwrap();
        sender.send(result).unwrap();
    }
}

use tokio::time;

use crate::prelude::*;

pub trait IsPublisher<T>: Send + Sync {
    fn publish_value(&self, value: T);
    fn finished_from_rust(&self);
}

#[uniffi::export(with_foreign)]
pub trait IsTransactionPublisher: IsPublisher<Transaction> {
    fn on_value(&self, value: Transaction);
    fn rust_is_subscribed_notify_cancellation_on(&self, listener: Arc<CancellationListener>);
    fn finished_from_rust_side(&self);
}

impl<U: IsTransactionPublisher> IsPublisher<Transaction> for U {
    fn publish_value(&self, value: Transaction) {
        self.on_value(value);
    }
    fn finished_from_rust(&self) {
        self.finished_from_rust_side()
    }
}

#[export]
impl GatewayClient {
    // Only marked `async` so that we are force to wrap it in a `Task {  }` Swift side
    // thus non blocking
    pub async fn subscribe_stream_of_latest_transactions(
        self: Arc<Self>, // must use `Arc<Self>` to not have to deal with tricky send/sync of self
        publisher: Arc<dyn IsTransactionPublisher>,
    ) {
        let (sender, receiver) = channel::<()>();

        let cancellation_listener = Arc::new(CancellationListener::new(sender));
        publisher.rust_is_subscribed_notify_cancellation_on(cancellation_listener);

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            tokio::select! {
                _ = async {
                    let mut last_tx_id: String = "".to_string();
                    loop {
                        let value = self.get_latest_transactions_or_panic().await;
                        if value.tx_id != last_tx_id {
                            // Only publish new, unique values
                            last_tx_id = value.tx_id.clone();
                            publisher.publish_value(value);
                        } else {
                            println!("🐌 Ignored duplicate value (no new TX done yet...)")
                        }
                        let delay = time::Duration::from_secs(5);
                        tokio::time::sleep(delay).await;
                    }
                } => {
                    // loop finished?
                }
                _ = async { receiver.await } => { println!("❌ RUST loop async fn received cancellation from Swift side => cancelling") }
            }
        });
    }
}
impl GatewayClient {
    pub async fn get_latest_transactions_or_panic(&self) -> Transaction {
        self.get_latest_transactions()
            .await
            .unwrap()
            .first()
            .expect("not to fail")
            .clone()
    }
}

////////
pub struct CancellationListenerInner {
    sender: Mutex<Option<Sender<()>>>,
}

impl CancellationListenerInner {
    pub(crate) fn new(sender: Sender<()>) -> Self {
        Self {
            sender: Mutex::new(Some(sender)),
        }
    }

    pub(crate) fn notify_cancelled(&self) {
        println!("❌ RUST received cancellation from Swift");
        self.sender
            .lock()
            .expect("Should only have access sender Mutex once.")
            .take()
            .expect("You MUST NOT call `notify_cancelled` twice in Swift.")
            .send(())
            .map_err(|_| RustSideError::FailedToPropagateResultFromFFIOperationBackToDispatcher)
            .expect("Must never fail, since some context's in FFI side cannot be throwing.")
    }
}
#[derive(Object)]
pub struct CancellationListener {
    cancellation_listener: CancellationListenerInner,
}
impl CancellationListener {
    pub(crate) fn new(sender: Sender<()>) -> Self {
        Self {
            cancellation_listener: CancellationListenerInner::new(sender),
        }
    }
}

#[export]
impl CancellationListener {
    fn notify_cancelled(&self) {
        self.cancellation_listener.notify_cancelled()
    }
}

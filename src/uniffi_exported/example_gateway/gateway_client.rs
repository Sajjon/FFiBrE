use crate::prelude::*;
use serde::de;
use std::borrow::Borrow;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use tokio::task;
use tokio::time;

/// A [Radix][https://www.radixdlt.com/] Gateway REST client, that makes its
/// network request using a "network antenna" 'installed' from FFI Side (Swift side).
#[derive(Object)]
pub struct GatewayClient {
    pub(crate) networking_dispatcher: FFIOperationDispatcher<FFINetworkingOutcomeListener>,
}

#[export]
impl GatewayClient {
    /// Constructs a new [`GatewayClient`] using a "network antenna" - a type
    /// implementing [`FFIOperationExecutor`] on the FFI side (Swift side), e.g.
    /// `[Swift]URLSession` which wraps the execution of a network call.
    #[uniffi::constructor]
    pub fn new(network_antenna: Arc<dyn FFINetworkingExecutor>) -> Self {
        Self {
            networking_dispatcher: FFIOperationDispatcher::<FFINetworkingOutcomeListener>::new(
                network_antenna,
            ),
        }
    }

    /// Reads the XRD balance of a Radix account with `[address]`, the actual
    /// network call is being done FFI Side (Swift side), but the parsing of JSON
    /// into models, and mapping of models [`GetEntityDetailsResponse`] ->
    /// balance (String).
    pub async fn get_xrd_balance_of_account(
        &self,
        address: String,
    ) -> Result<String, FFIBridgeError> {
        self.post(
            "state/entity/details",
            GetEntityDetailsRequest::new(address),
            parse_xrd_balance_from,
        )
        .await
    }

    pub fn subscribe_stream_of_latest_transactions(
        self: Arc<Self>,
        publisher: Arc<dyn IsTransactionPublisher>,
    ) {
        let cancelled = Arc::new(AtomicBool::new(false));
        let was_cancelled = cancelled.clone();
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let mut last_tx_id: String = "".to_string();
                loop {
                    if was_cancelled.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }
                    let value = self.halt_and_catch_fire_get_latest_transactions().await;
                    if value.tx_id != last_tx_id {
                        // Only publish new, unique values
                        last_tx_id = value.tx_id.clone();
                        publisher.publish_value(value);
                    }
                    let delay = time::Duration::from_secs(5);
                    tokio::time::sleep(delay).await;
                }
            });
        println!("subscribe_stream_of_latest_transactions ENDED");
    }

    pub async fn halt_and_catch_fire_get_latest_transactions(&self) -> Transaction {
        self.get_latest_transactions()
            .await
            .unwrap()
            .first()
            .unwrap()
            .clone()
    }
    pub async fn get_latest_transactions(&self) -> Result<Vec<Transaction>, FFIBridgeError> {
        self.post(
            "stream/transactions",
            GetTransactionStreamRequest::default(),
            parse_transactions,
        )
        .await
    }
}

pub trait IsPublisher<T>: Send + Sync {
    fn publish_value(&self, value: T);
    fn cancel_subscription(&self);
}

#[uniffi::export(with_foreign)]
pub trait IsTransactionPublisher: IsPublisher<Transaction> {
    fn on_value(&self, value: Transaction);
    fn cancel(&self);
}

impl<U: IsTransactionPublisher> IsPublisher<Transaction> for U {
    fn publish_value(&self, value: Transaction) {
        self.on_value(value);
    }
    fn cancel_subscription(&self) {
        self.cancel()
    }
}

impl GatewayClient {
    fn model_from_response<U>(&self, response: FFINetworkingResponse) -> Result<U, RustSideError>
    where
        U: for<'a> Deserialize<'a>,
    {
        if let 200..=299 = response.status_code {
            // all good
        } else {
            return Err(RustSideError::BadResponseCode);
        }

        let body = response.body;
        if body.is_empty() {
            return Err(RustSideError::ResponseBodyWasNil.into());
        }

        serde_json::from_slice::<U>(&body).map_err(|_| {
            RustSideError::UnableJSONDeserializeHTTPResponseBodyIntoTypeName {
                type_name: std::any::type_name::<U>().to_owned(),
            }
        })
    }

    async fn make_request<T, U, V, F, E>(
        &self,
        path: impl AsRef<str>,
        method: impl AsRef<str>,
        request: T,
        map: F,
    ) -> Result<V, FFIBridgeError>
    where
        T: Serialize,
        U: for<'a> Deserialize<'a>,
        F: Fn(U) -> Result<V, E>,
        E: Into<FFIBridgeError>,
    {
        // JSON serialize request into body bytes
        let body = to_vec(&request).unwrap();

        // Append relative path to base url
        let url = format!("https://mainnet.radixdlt.com/{}", path.as_ref());

        // Create Network request object, which will be translated by
        // Swift side into a `[Swift]URLRequest`
        let request = FFINetworkingRequest {
            url,
            body,
            method: method.as_ref().to_owned(),
            headers: HashMap::<String, String>::from_iter([(
                "Content-Type".to_owned(),
                "application/json".to_owned(),
            )]),
        };

        // Let Swift side make network request and await response
        let response = self.networking_dispatcher.dispatch(request).await?;

        // Read out HTTP body from response and JSON parse it into U
        let model = self
            .model_from_response(response)
            .map_err(|error| FFIBridgeError::FromRust { error })?;

        // Map U -> V
        map(model).map_err(|e| e.into())
    }

    pub(crate) async fn post<T, U, V, F, E>(
        &self,
        path: impl AsRef<str>,
        request: T,
        map: F,
    ) -> Result<V, FFIBridgeError>
    where
        T: Serialize,
        U: for<'a> Deserialize<'a>,
        F: Fn(U) -> Result<V, E>,
        E: Into<FFIBridgeError>,
    {
        self.make_request(path, "POST", request, map).await
    }
}

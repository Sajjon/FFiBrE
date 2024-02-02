use serde::{Deserialize, Serialize};
use serde_json::to_vec;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::oneshot;

#[derive(uniffi::Record, Clone, Debug)]
pub struct NetworkRequest {
    url: String,
    body: Vec<u8>,
    method: String,
    headers: HashMap<String, String>,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct NetworkResponse {
    response_code: u16,
    body: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Clone, thiserror::Error, uniffi::Error)]
pub enum NetworkError {
    #[error("Bad code")]
    BadResponseCode,

    #[error("No XRD balance found in entity state response")]
    NoXRDBalanceFound,

    #[error("Failed to receive response from Swift")]
    FailedToReceiveResponseFromSwift,

    #[error("URLSession data task request failed, underlying error: {underlying}")]
    URLSessionDataTaskFailed { underlying: String },

    #[error("Unable to JSON deserialize HTTP response body into type: {type_name}")]
    UnableJSONDeserializeHTTPResponseBodyIntoTypeName { type_name: String },
}

#[uniffi::export]
pub trait NotifyRustFromSwift: Send + Sync {
    fn response(&self, result: NetworkResult);
}

#[uniffi::export(with_foreign)]
pub trait HTTPClientRequestSender: Send + Sync {
    fn send(
        &self,
        request: NetworkRequest,
        response_back: Arc<dyn NotifyRustFromSwift>,
    ) -> Result<(), NetworkError>;
}

#[derive(uniffi::Enum, Clone, Debug)]
pub enum NetworkResult {
    Success { value: NetworkResponse },
    Failure { error: NetworkError },
}
impl From<NetworkResult> for Result<NetworkResponse, NetworkError> {
    fn from(value: NetworkResult) -> Self {
        match value {
            NetworkResult::Success { value } => Ok(value),
            NetworkResult::Failure { error } => Err(error),
        }
    }
}

const XRD: &str = "resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd";

#[derive(Deserialize, Clone)]
struct FungibleResourceItem {
    amount: String,
    resource_address: String,
}

#[derive(Deserialize, Clone)]
struct FungibleResources {
    items: Vec<FungibleResourceItem>,
}

#[derive(Deserialize, Clone)]
struct EntityStateItem {
    fungible_resources: FungibleResources,
}

#[derive(Deserialize, Clone)]
struct EntityState {
    items: Vec<EntityStateItem>,
}

fn parse_xrd_balance_from(entity_state: EntityState) -> Result<String, NetworkError> {
    assert_eq!(entity_state.items.len(), 1);
    let item: &EntityStateItem = entity_state.items.first().unwrap();
    let fungible_resources = item.fungible_resources.clone();

    fungible_resources
        .items
        .into_iter()
        .filter(|x| x.resource_address == XRD)
        .map(|x| x.amount.clone())
        .next()
        .ok_or(NetworkError::NoXRDBalanceFound)
}

#[derive(Serialize)]
struct GetEntityDetailsRequest {
    addresses: Vec<String>,
}
impl GetEntityDetailsRequest {
    pub fn new(address: impl AsRef<str>) -> Self {
        Self {
            addresses: vec![address.as_ref().to_owned()],
        }
    }
}

#[derive(uniffi::Object)]
pub struct HTTPClient {
    request_sender: Arc<dyn HTTPClientRequestSender>,
}

#[uniffi::export]
impl HTTPClient {
    #[uniffi::constructor]
    pub fn new(request_sender: Arc<dyn HTTPClientRequestSender>) -> Self {
        Self { request_sender }
    }
}

impl HTTPClient {
    async fn make_request(&self, request: NetworkRequest) -> Result<NetworkResponse, NetworkError> {
        println!("HTTPClient - make_request START");
        let (response_sender, response_receiver) = oneshot::channel();
        let sender_wrapper = OneshotSenderWrapper::new(response_sender);
        println!("HTTPClient - make_request calling self.request_sender.send");
        self.request_sender
            .send(request, Arc::new(sender_wrapper))
            .unwrap();

        println!("HTTPClient - make_request calling response_receiver.await");
        response_receiver
            .await
            .map_err(|_| NetworkError::FailedToReceiveResponseFromSwift)
            .and_then(|r| r.into())
    }
}

#[derive(uniffi::Object)]
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

#[derive(uniffi::Object)]
pub struct GatewayClient {
    http_client: Arc<HTTPClient>,
}
impl GatewayClient {
    async fn make_request<T, U, V, F>(
        &self,
        path: impl AsRef<str>,
        method: impl AsRef<str>,
        request: T,
        map: F,
    ) -> Result<V, NetworkError>
    where
        T: Serialize,
        U: for<'a> Deserialize<'a>,
        F: Fn(U) -> Result<V, NetworkError>,
    {
        println!("GatewayClient make_request START");
        let body = to_vec(&request).unwrap();
        let url = format!("https://mainnet.radixdlt.com/{}", path.as_ref());
        let request = NetworkRequest {
            url,
            body,
            method: method.as_ref().to_owned(),
            headers: HashMap::<String, String>::from_iter([(
                "Content-Type".to_owned(),
                "application/json".to_owned(),
            )]),
        };
        println!("GatewayClient make_request calling http_client.make_request");
        let response = self.http_client.make_request(request).await;
        println!("GatewayClient make_request calling http_client.make_request DONE??");
        response
            .and_then(|r| {
                serde_json::from_slice::<U>(&r.body).map_err(|_| {
                    NetworkError::UnableJSONDeserializeHTTPResponseBodyIntoTypeName {
                        type_name: std::any::type_name::<U>().to_owned(),
                    }
                })
            })
            .and_then(|s| map(s))
    }

    async fn post<T, U, V, F>(
        &self,
        path: impl AsRef<str>,
        request: T,
        map: F,
    ) -> Result<V, NetworkError>
    where
        T: Serialize,
        U: for<'a> Deserialize<'a>,
        F: Fn(U) -> Result<V, NetworkError>,
    {
        self.make_request(path, "POST", request, map).await
    }
}

#[uniffi::export]
impl GatewayClient {
    #[uniffi::constructor]
    pub fn new(http_client: Arc<HTTPClient>) -> Self {
        Self { http_client }
    }

    pub async fn get_xrd_balance_of_account(
        &self,
        address: String,
    ) -> Result<String, NetworkError> {
        self.post(
            "state/entity/details",
            GetEntityDetailsRequest::new(address),
            parse_xrd_balance_from,
        )
        .await
    }
}

uniffi::include_scaffolding!("network");

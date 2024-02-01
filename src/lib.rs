use serde::{Deserialize, Serialize};
use serde_json::to_vec;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

#[derive(uniffi::Record, Clone, Debug)]
struct NetworkRequest {
    id: String,
    url: String,
    body: Vec<u8>,
    method: String,
    headers: HashMap<String, String>,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct NetworkResponse {
    id: String,
    response_code: u16,
    body: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Clone, thiserror::Error, uniffi::Error)]
pub enum NetworkError {
    #[error("Bad code")]
    BadResponseCode,

    #[error("No XRD balance found in entity state response")]
    NoXRDBalanceFound,

    #[error("Unable to JSON deserialize HTTP response body into type: {type_name}")]
    UnableJSONDeserializeHTTPResponseBodyIntoTypeName { type_name: String },
}

#[uniffi::export]
pub trait HTTPClientRequestSender: Send + Sync {
    /// Called by Rust, Swift side makes call, and then
    /// Swift side SHOULD call `got_response` method
    /// on `HTTPClient`
    fn send(&self, request: NetworkRequest) -> Result<(), NetworkError>;
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

#[derive(uniffi::Record, Clone, Debug)]
pub struct ResultOfNetworkRequest {
    request: NetworkRequest,
    result: NetworkResult,
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

use tokio::sync::Semaphore;

#[derive(Debug)]
pub struct Context {
    pub semaphore: Semaphore,
    pub result: Arc<Mutex<Option<ResultOfNetworkRequest>>>,
}
impl Context {
    fn new() -> Self {
        Self {
            semaphore: Semaphore::new(0),
            result: Arc::new(Mutex::new(None)),
        }
    }
    fn global() -> &'static Self {
        CTX.get().expect("Context is not initialized")
    }

    async fn await_response() -> Result<NetworkResponse, NetworkError> {
        let ctx = Self::global();
        drop(ctx.semaphore.acquire().await.unwrap());
        let res: Result<NetworkResponse, NetworkError> =
            ctx.result.lock().unwrap().clone().unwrap().result.into();
        *ctx.result.lock().unwrap() = None;
        res
    }

    fn got_result(result: ResultOfNetworkRequest) {
        let ctx = Self::global();
        *ctx.result.lock().unwrap() = Some(result);
        ctx.semaphore.add_permits(1)
    }
}

use once_cell::sync::OnceCell;
static CTX: OnceCell<Context> = OnceCell::new();

#[uniffi::export]
impl HTTPClient {
    #[uniffi::constructor]
    pub fn new(request_sender: Arc<dyn HTTPClientRequestSender>) -> Self {
        Self { request_sender }
    }

    pub fn got_result_of_network_request(&self, result: ResultOfNetworkRequest) {
        Context::got_result(result)
    }
}

impl HTTPClient {
    async fn make_request(&self, request: NetworkRequest) -> Result<NetworkResponse, NetworkError> {
        self.request_sender.send(request).unwrap();
        Context::await_response().await
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
        let body = to_vec(&request).unwrap();
        let url = format!("https://mainnet.radixdlt.com/{}", path.as_ref());
        let request = NetworkRequest {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            body,
            method: method.as_ref().to_owned(),
            headers: HashMap::<String, String>::from_iter([(
                "Content-Type".to_owned(),
                "application/json".to_owned(),
            )]),
        };
        let response = self.http_client.make_request(request).await;
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

    async fn get<T, U, V, F>(
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
        self.make_request(path, "GET", request, map).await
    }
}

#[uniffi::export]
impl GatewayClient {
    #[uniffi::constructor]
    pub fn new(http_client: Arc<HTTPClient>) -> Self {
        CTX.set(Context::new()).unwrap();
        Self { http_client }
    }

    pub async fn get_xrd_balance_of_account(
        &self,
        address: String,
    ) -> Result<String, NetworkError> {
        self.get(
            "state/entity/details",
            GetEntityDetailsRequest::new(address),
            parse_xrd_balance_from,
        )
        .await
    }
}

uniffi::include_scaffolding!("network");

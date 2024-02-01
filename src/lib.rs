use serde::{Deserialize, Serialize};
use serde_json::to_vec;
use std::{collections::HashMap, os::unix::thread, sync::Arc};

#[uniffi::export]
pub trait NetworkRequest: Send + Sync {
    fn url(&self) -> String;
    fn http_body(&self) -> Vec<u8>;
    fn http_method(&self) -> String;
    fn http_header_fields(&self) -> HashMap<String, String>;
}

#[derive(uniffi::Record)]
struct NetworkRequestImpl {
    url: String,
    body: Vec<u8>,
    method: String,
    headers: HashMap<String, String>,
}
impl NetworkRequest for NetworkRequestImpl {
    fn url(&self) -> String {
        self.url.to_string()
    }

    fn http_body(&self) -> Vec<u8> {
        self.body.clone()
    }

    fn http_method(&self) -> String {
        self.method.clone()
    }

    fn http_header_fields(&self) -> HashMap<String, String> {
        self.headers.clone()
    }
}

#[uniffi::export]
pub trait NetworkResponse: NetworkRequest {
    fn status_code(&self) -> u16;
}

#[derive(uniffi::Record, Clone)]
pub struct NetworkResponseImpl {}
impl NetworkRequest for NetworkResponseImpl {
    fn url(&self) -> String {
        todo!()
    }

    fn http_body(&self) -> Vec<u8> {
        todo!()
    }

    fn http_method(&self) -> String {
        todo!()
    }

    fn http_header_fields(&self) -> HashMap<String, String> {
        todo!()
    }
}
impl NetworkResponse for NetworkResponseImpl {
    fn status_code(&self) -> u16 {
        todo!()
    }
}
impl NetworkResponseImpl {
    pub fn new() -> Self {
        todo!()
    }
}
impl NetworkRequestImpl {
    pub fn new(url: String) -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, thiserror::Error, uniffi::Error)]
pub enum Error {
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
    fn send(&self, request: NetworkRequestImpl) -> Result<(), Error>;
}

#[derive(uniffi::Enum, Clone)]
pub enum NetworkResult {
    Success { value: NetworkResponseImpl },
    Failure { error: Error },
}

#[uniffi::export]
pub trait HTTPClientNetworkResultReceiver: Send + Sync {
    /// Called by Rust, Swift side makes call, and then
    /// Swift side SHOULD call `got_result` method
    /// on `HTTPClient`
    fn got_result(&self, result: NetworkResult, for_request: NetworkRequestImpl);
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

fn parse_xrd_balance_from(entity_state: EntityState) -> Result<String, Error> {
    assert_eq!(entity_state.items.len(), 1);
    let item: &EntityStateItem = entity_state.items.first().unwrap();
    let fungible_resources = item.fungible_resources.clone();

    fungible_resources
        .items
        .into_iter()
        .filter(|x| x.resource_address == XRD)
        .map(|x| x.amount.clone())
        .next()
        .ok_or(Error::NoXRDBalanceFound)
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

// #[derive(uniffi::Object)]
pub struct HTTPClient {
    // sender: tokio::sync::mpsc::UnboundedSender<NetworkRequestImpl>,
    // receiver: tokio::sync::mpsc::UnboundedReceiver<NetworkResult>,
    request_sender: Arc<dyn HTTPClientRequestSender>,
    result_receiver: Arc<dyn HTTPClientNetworkResultReceiver>, // sender: std::sync::mpsc::SyncSender<NetworkRequestImpl>,
                                                               // receiver: std::sync::mpsc::Receiver<NetworkResult>
}

// #[uniffi::export]
impl HTTPClient {
    // #[uniffi::constructor]
    pub fn new(
        request_sender: Arc<dyn HTTPClientRequestSender>,
        result_receiver: Arc<dyn HTTPClientNetworkResultReceiver>,
    ) -> Self {
        // let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

        // let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        // Self { receiver, sender }
        Self {
            request_sender,
            result_receiver,
        }
    }
}

impl HTTPClient {
    async fn make_request(
        &self,
        request: NetworkRequestImpl,
    ) -> Result<NetworkResponseImpl, Error> {
        // self.request_sender.send(request);
        // self.sender.send(request).unwrap();
        // loop {
        //     self.receiver.recv()
        // }
        self.request_sender.send(request).unwrap();

        self.result_receiver.
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
    ) -> Result<V, Error>
    where
        T: Serialize,
        U: for<'a> Deserialize<'a>,
        F: Fn(U) -> Result<V, Error>,
    {
        let body = to_vec(&request).unwrap();
        let url = format!("https://mainnet.radixdlt.com/{}", path.as_ref());
        let request = NetworkRequestImpl {
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
                serde_json::from_slice::<U>(&r.http_body()).map_err(|_| {
                    Error::UnableJSONDeserializeHTTPResponseBodyIntoTypeName {
                        type_name: std::any::type_name::<U>().to_owned(),
                    }
                })
            })
            .and_then(|s| map(s))
    }

    async fn get<T, U, V, F>(&self, path: impl AsRef<str>, request: T, map: F) -> Result<V, Error>
    where
        T: Serialize,
        U: for<'a> Deserialize<'a>,
        F: Fn(U) -> Result<V, Error>,
    {
        self.make_request(path, "GET", request, map).await
    }
}

impl GatewayClient {
    pub fn new_with_http_client(http_client: Arc<HTTPClient>) -> Self {
        Self { http_client }
    }
}

#[uniffi::export]
impl GatewayClient {
  
    #[uniffi::constructor]
    pub fn new(request_sender: Arc<dyn HTTPClientRequestSender>, result_receiver: Arc<dyn HTTPClientNetworkResultReceiver>) -> Self {
        Self::new_with_http_client(HTTPClient::new(request_sender, result_receiver).into())
    }

    pub async fn get_xrd_balance_of_account(&self, address: String) -> Result<String, Error> {
        self.get(
            "state/entity/details",
            GetEntityDetailsRequest::new(address),
            parse_xrd_balance_from,
        )
        .await
    }
}

uniffi::include_scaffolding!("network");

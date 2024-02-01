use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::to_vec;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Semaphore, TryAcquireError};
use url::Url;

#[uniffi::export]
pub trait NetworkRequest: Send + Sync {
    fn url(&self) -> String;
    fn http_body(&self) -> Vec<u8>;
    fn http_method(&self) -> String;
    fn http_header_fields(&self) -> HashMap<String, String>;
}

struct NetworkRequestImpl {
    url: Url,
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

#[derive(Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum Error {
    #[error("Bad code")]
    BadResponseCode,

    #[error("No XRD balance found in entity state response")]
    NoXRDBalanceFound,

    #[error("Failed to parse decimal from amount string {0}")]
    FailedToParseDecimalFromAmountString { amount: String },
}

#[uniffi::export]
pub trait GotNetworkResponse: Send + Sync {
    fn provide_response(&self, response: Arc<dyn NetworkResponse>);
}

type Callback = fn(Arc<dyn NetworkResponse>);
struct GotNetworkResponseImpl {
    callback: Callback,
}
impl GotNetworkResponse for GotNetworkResponseImpl {
    fn provide_response(&self, response: Arc<dyn NetworkResponse>) {
        (self.callback)(response)
    }
}

#[uniffi::export]
pub trait NetworkRequestMaker: Send + Sync {
    fn make_request(
        &self,
        request: Arc<dyn NetworkRequest>,
        on_response: Arc<dyn GotNetworkResponse>,
    ) -> Result<(), Error>;
}

const xrd_resource_address: String =
    "resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd";

#[derive(Deserialize)]
struct FungibleResourceItem {
    amount: String,
    resource_address: String,
}

#[derive(Deserialize)]
struct FungibleResources {
    items: Vec<FungibleResourceItem>,
}

#[derive(Deserialize)]
struct EntityStateItem {
    fungible_resources: FungibleResources,
}

#[derive(Deserialize)]
struct EntityState {
    items: Vec<EntityStateItem>,
}

fn parse_xrd_balance_from(entity_state: EntityState) -> Result<Decimal, Error> {
    assert_eq!(entity_state.items.len(), 1);
    let item: &EntityStateItem = entity_state.items.first().unwrap();
    let fungible_resources = item.fungible_resources;

    fungible_resources
        .items
        .iter()
        .filter(|x| x.resource_address == xrd_resource_address)
        .map(|x| x.amount)
        .next()
        .ok_or(Error::NoXRDBalanceFound)
        .and_then(|s| {
            s.parse::<Decimal>()
                .map_err(|_| Error::FailedToParseDecimalFromAmountString { amount: s.clone() })
        })
}

#[derive(Serialize)]
struct GetEntityDetailsRequest {
    addresses: Vec<String>,
}

// curl -X POST https://mainnet.radixdlt.com/state/entity/details -H 'Content-Type: application/json' -d '{"addresses": ["account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease"]}'
#[uniffi::export]
pub async fn get_xrd_balance_of_account(
    address: String,
    request_maker: Arc<dyn NetworkRequestMaker>,
) -> Result<String, Error> {
    let body_request = GetEntityDetailsRequest {
        addresses: vec![address],
    };
    let body = to_vec(&body_request).unwrap();
    let request = NetworkRequestImpl {
        url: "https://mainnet.radixdlt.com/state/entity/details"
            .parse()
            .unwrap(),
        body,
        method: "POST".to_string(),
        headers: HashMap::<String, String>::from_iter([(
            "Content-Type".to_owned(),
            "application/json".to_owned(),
        )]),
    };

    let response_listener = GotNetworkResponseImpl {
        callback: |response| println!("Got response - status code: {}", response.status_code()),
    };

    request_maker.make_request(Arc::new(request), Arc::new(response_listener));

    todo!("how to blocking wait for `callback` in `GotNetworkResponseImpl`? We need a 'promise handle' basically...")
}

uniffi::include_scaffolding!("network");

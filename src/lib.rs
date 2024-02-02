mod gateway_client;
mod http_client;
mod network_error;

pub mod prelude {
    pub use crate::gateway_client::*;
    pub use crate::http_client::*;
    pub use crate::network_error::*;

    pub(crate) use serde::{Deserialize, Serialize};
    pub(crate) use serde_json::to_vec;
    pub(crate) use std::collections::HashMap;
    pub(crate) use std::sync::{Arc, Mutex};
    pub(crate) use uniffi::{export, Enum, Error, Object, Record, include_scaffolding};
}

pub use prelude::*;

include_scaffolding!("network");

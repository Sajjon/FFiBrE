mod internal;
mod ffibre_error;
mod uniffi_exported;

pub mod prelude {
    pub(crate) use crate::internal::*;
    pub use crate::ffibre_error::*;
    pub use crate::uniffi_exported::*;

    pub(crate) use serde::{Deserialize, Serialize};
    pub(crate) use serde_json::to_vec;
    pub(crate) use std::collections::HashMap;
    pub(crate) use std::sync::{Arc, Mutex};
    pub(crate) use tokio::sync::oneshot::{channel, Sender};
    pub(crate) use uniffi::{export, include_scaffolding, Enum, Error, Object, Record};
}

pub use prelude::*;

include_scaffolding!("ffibre");

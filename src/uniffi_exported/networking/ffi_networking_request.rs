use crate::prelude::*;

/// An abstraction of a HTTP Network Request to be made FFI Side (Swift side),
/// e.g. by URLSession in Swift.
#[derive(Record, Clone, Debug, PartialEq, Eq)]
pub struct FFINetworkingRequest {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,

    pub body: Vec<u8>,
}

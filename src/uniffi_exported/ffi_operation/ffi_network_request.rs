use crate::prelude::*;

/// An abstraction of a HTTP Network Request to be made FFI Side (Swift side),
/// e.g. by URLSession in Swift. 
#[derive(Record, Clone, Debug, PartialEq, Eq)]
pub struct NetworkRequest {
    pub url: String,
    pub body: Vec<u8>,
    pub method: String,
    pub headers: HashMap<String, String>,
}

use crate::prelude::*;

/// An abstraction of a HTTP Network Response the FFI Side (Swift side),
/// completed a [`NetworkRequest`] with
#[derive(Record, Clone, Debug, PartialEq, Eq)]
pub struct NetworkResponse {
    pub status_code: u16,

    pub url: String,
    pub headers: HashMap<String, String>,
    
    /// Can be empty though.
    pub body: Vec<u8>,
}

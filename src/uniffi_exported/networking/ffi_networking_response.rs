use crate::prelude::*;

/// An abstraction of a HTTP Network Response the FFI Side (Swift side),
/// completed a [`FFINetworkingRequest`] with
#[derive(Record, Clone, Debug, PartialEq, Eq)]
pub struct FFINetworkingResponse {
    pub status_code: u16,

    /// Can be empty.
    pub body: Vec<u8>,
}

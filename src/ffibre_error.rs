use crate::prelude::*;
use thiserror::Error as ThisError;

#[derive(Debug, PartialEq, Eq, Clone, ThisError, Error)]
pub enum FFISideError {
    #[error(transparent)]
    Networking {
        #[from]
        error: FFINetworkingError,
    },

    #[error(transparent)]
    FileIOWrite {
        #[from]
        error: FFIFileIOWriteError,
    },

    #[error(transparent)]
    FileIORead {
        #[from]
        error: FFIFileIOReadError,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, ThisError, Error)]
pub enum RustSideError {
    #[error("No response code")]
    NoResponseCode,

    #[error("Bad response code")]
    BadResponseCode,

    #[error("Unable to JSON deserialize HTTP response body into type: {type_name}")]
    UnableJSONDeserializeHTTPResponseBodyIntoTypeName { type_name: String },

    #[error("No XRD balance found in entity state response")]
    NoXRDBalanceFound,

    #[error("Failed to receive response from Swift")]
    FailedToReceiveResponseFromSwift,

    #[error("Failed to propagate FFI operation result back to dispatcher")]
    FailedToPropagateResultFromFFIOperationBackToDispatcher,

    #[error("HTTP Body of response from Swift was nil")]
    ResponseBodyWasNil,

    #[error("Wrong response kind from FFIOperationOk, expected NetworkResponse")]
    WrongFFIOperationOKExpectedNetworkResponse,
}

#[derive(Debug, PartialEq, Eq, Clone, ThisError, Error)]
pub enum FFIBridgeError {
    #[error(transparent)]
    FromRust {
        #[from]
        error: RustSideError,
    },

    #[error(transparent)]
    FromFFI {
        #[from]
        error: FFISideError,
    },
}

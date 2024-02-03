use crate::prelude::*;
use thiserror::Error as ThisError;

#[derive(Debug, PartialEq, Eq, Clone, ThisError, Error)]
pub enum SwiftSideError {
    #[error("Fail to create Swift 'Foundation.URL' from string: '{string}'")]
    FailedToCreateURLFrom { string: String },

    #[error(
        "Swift 'URLRequest' failed with code '{:?}', error message from Gateway: '{:?}', underlying error (URLSession): '{:?}'",
        status_code,
        error_message_from_gateway,
        url_session_underlying_error
    )]
    RequestFailed {
        status_code: Option<u16>,
        url_session_underlying_error: Option<String>,
        error_message_from_gateway: Option<String>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, ThisError, Error)]
pub enum RustSideError {
    #[error("No response code")]
    NoResponseCode,

    #[error("Bad response code")]
    BadResponseCode,

    #[error(
        "Tried to dispatch unsupported operation {:?}, handler only supports: {:?}",
        operation,
        only_supported
    )]
    UnsupportedOperation {
        operation: FFIOperation,
        only_supported: Vec<FFIOperationKind>,
    },

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
pub enum NetworkError {
    #[error(transparent)]
    FromRust {
        #[from]
        error: RustSideError,
    },

    #[error(transparent)]
    FromSwift {
        #[from]
        error: SwiftSideError,
    },
}

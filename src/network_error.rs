use crate::prelude::*;
use thiserror::Error as ThisError;

#[derive(Debug, PartialEq, Eq, Clone, ThisError, Error)]
pub enum SwiftSideError {
    #[error("Fail to create Swift 'Foundation.URL' from string: '{string}'")]
    FailedToCreateURLFrom { string: String },

    #[error("Unable to cast Swift 'Foundation.URLResponse' into 'Foundation.HTTPURLResponse'")]
    UnableToCastUrlResponseToHTTPUrlResponse,

    #[error(
        "Swift 'URLRequest' failed with code '{status_code}', error message from Gateway: '{:?}', underlying error (URLSession): '{:?}'",
        error_message_from_gateway,
        url_session_underlying_error
    )]
    RequestFailed {
        status_code: u16,
        url_session_underlying_error: Option<String>,
        error_message_from_gateway: Option<String>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, ThisError, Error)]
pub enum RustSideError {
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

    #[error("Failed to propagate FFI operation result back to displatcher")]
    FailedToPropagateResultFromFFIOperationBackToDispatcher,

    #[error("HTTP Body of response from Swift was nil")]
    ResponseBodyWasNil,
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

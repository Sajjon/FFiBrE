use crate::prelude::*;
use thiserror::Error as ThisError;

#[derive(Debug, PartialEq, Eq, Clone, ThisError, Error)]
pub enum SwiftSideError {
    #[error("Fail to create Swift 'Foundation.URL' from string: '{string}'")]
    FailedToCreateURLFrom { string: String },

    #[error("Unable to cast Swift 'Foundation.URLResponse' into 'Foundation.HTTPURLResponse'")]
    UnableToCastUrlResponseToHTTPUrlResponse,

    #[error("Swift 'URLRequest' failed with code '{status_code}', reason: '{reason}'")]
    RequestFailed { status_code: u16, reason: String },
}

#[derive(Debug, PartialEq, Eq, Clone, ThisError, Error)]
pub enum RustSideError {
    #[error("Unable to JSON deserialize HTTP response body into type: {type_name}")]
    UnableJSONDeserializeHTTPResponseBodyIntoTypeName { type_name: String },

    #[error("No XRD balance found in entity state response")]
    NoXRDBalanceFound,

    #[error("Failed to receive response from Swift")]
    FailedToReceiveResponseFromSwift,

    #[error("HTTP Body of response from Swift was nil")]
    ResponseBodyWasNil,

    #[error("HTTP Body of response from Swift was empty")]
    ResponseBodyWasEmpty,
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

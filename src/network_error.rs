use crate::prelude::*;
use thiserror::Error as ThisError;

#[derive(Debug, PartialEq, Eq, Clone, ThisError, Error)]
pub enum NetworkError {
    #[error("Bad code")]
    BadResponseCode,

    #[error("No XRD balance found in entity state response")]
    NoXRDBalanceFound,

    #[error("Failed to receive response from Swift")]
    FailedToReceiveResponseFromSwift,

    #[error("URLSession data task request failed, underlying error: {underlying}")]
    URLSessionDataTaskFailed { underlying: String },

    #[error("Unable to JSON deserialize HTTP response body into type: {type_name}")]
    UnableJSONDeserializeHTTPResponseBodyIntoTypeName { type_name: String },
}

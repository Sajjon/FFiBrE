use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Enum)]
pub enum FFIOperationKind {
    Networking,
}

pub trait HasFFIOperationKindInstance {
    fn operation_kind(&self) -> FFIOperationKind;
}

impl HasFFIOperationKindInstance for NetworkRequest {
    fn operation_kind(&self) -> FFIOperationKind {
        FFIOperationKind::Networking
    }
}

/// An operation we dispatch from Rust side to FFI (Swift)
/// side using [`FFIOperationDispatcher`], which responds
/// back to us (Rust side) using [`FFIDataResultListener`]
/// callback.
#[derive(Clone, Debug, PartialEq, Eq, Enum)]
pub enum FFIOperation {
    Networking { request: NetworkRequest },
}
impl HasFFIOperationKindInstance for FFIOperation {
    fn operation_kind(&self) -> FFIOperationKind {
        match self {
            FFIOperation::Networking { request: _ } => FFIOperationKind::Networking,
        }
    }
}

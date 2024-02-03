use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FFIOperationKind {
    Networking,
    FileIOWrite,
}

pub trait HasFFIOperationKindInstance {
    fn operation_kind(&self) -> FFIOperationKind;
}

impl HasFFIOperationKindInstance for NetworkRequest {
    fn operation_kind(&self) -> FFIOperationKind {
        FFIOperationKind::Networking
    }
}

#[derive(Clone, Debug, PartialEq, Eq, enum_as_inner::EnumAsInner)]
pub enum FFIOperation {
    Networking { request: NetworkRequest },
    FileIOWrite { request: FFIFileIOWriteRequest },
}

impl HasFFIOperationKindInstance for FFIOperation {
    fn operation_kind(&self) -> FFIOperationKind {
        match self {
            FFIOperation::Networking { request: _ } => FFIOperationKind::Networking,
            FFIOperation::FileIOWrite { request: _ } => FFIOperationKind::FileIOWrite,
        }
    }
}

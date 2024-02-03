use crate::prelude::*;

/// The `Ok`` value of a successful [`FfiOperationResult`],
/// passed from Ffi for some [`FfiOperation`] send from Rust side
/// to Fff side (Swift side).
#[derive(Clone, Debug, PartialEq, Eq, enum_as_inner::EnumAsInner)]
pub enum FFIOperationOk {
    Networking { response: NetworkResponse },
    FileIOWrite { response: FFIFileIOWriteResponse },
}

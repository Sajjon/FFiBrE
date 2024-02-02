use crate::prelude::*;

/// UniFFI does not allow us to pass `[Swift]Result` <-> `[Rust]Result` (I think),
/// so this is an implementation of that, so that a [`FFIOperationHandler`] can pass
/// the result of an [`FFIOperation`] back to Rust land, i.e. either a `Failure`
/// or a `Success` (data).
#[derive(Enum, Clone, Debug)]
pub enum FFIOperationResult {
    Success { value: Option<Vec<u8>> },
    Failure { error: SwiftSideError },
}

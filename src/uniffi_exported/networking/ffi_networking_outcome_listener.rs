use crate::prelude::*;

#[derive(Object)]
pub struct FFINetworkingOutcomeListener {
    result_listener: FFIOperationOutcomeListener<FFINetworkingOutcome>,
}
impl IsOutcomeListener for FFINetworkingOutcomeListener {
    type Request = FFINetworkingRequest;
    type Response = FFINetworkingResponse;
    type Failure = FFINetworkingError;
    type Outcome = FFINetworkingOutcome;
}

impl From<FFIOperationOutcomeListener<FFINetworkingOutcome>> for FFINetworkingOutcomeListener {
    fn from(value: FFIOperationOutcomeListener<FFINetworkingOutcome>) -> Self {
        Self::with_result_listener(value)
    }
}
impl FFINetworkingOutcomeListener {
    pub fn with_result_listener(
        result_listener: FFIOperationOutcomeListener<FFINetworkingOutcome>,
    ) -> Self {
        Self { result_listener }
    }
}

#[export]
impl FFINetworkingOutcomeListener {
    fn notify_outcome(&self, result: FFINetworkingOutcome) {
        self.result_listener.notify_outcome(result.into())
    }
}

////////
pub struct CancellationListenerInner {
    sender: Mutex<Option<Sender<()>>>,
}

impl CancellationListenerInner {
    pub(crate) fn new(sender: Sender<()>) -> Self {
        Self {
            sender: Mutex::new(Some(sender)),
        }
    }

    pub(crate) fn notify_cancelled(&self) {
        println!("❌ RUST Notified cancelled");
        self.sender
            .lock()
            .expect("Should only have access sender Mutex once.")
            .take()
            .expect("You MUST NOT call `notify_cancelled` twice in Swift.")
            .send(())
            .map_err(|_| RustSideError::FailedToPropagateResultFromFFIOperationBackToDispatcher)
            .expect("Must never fail, since some context's in FFI side cannot be throwing.")
    }
}
#[derive(Object)]
pub struct CancellationListener {
    cancellation_listener: CancellationListenerInner,
}
impl CancellationListener {
    pub(crate) fn new(sender: Sender<()>) -> Self {
        Self {
            cancellation_listener: CancellationListenerInner::new(sender),
        }
    }
}

#[export]
impl CancellationListener {
    fn notify_cancelled(&self) {
        self.cancellation_listener.notify_cancelled()
    }
}

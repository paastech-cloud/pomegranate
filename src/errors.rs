use std::boxed::Box;

/// # Application errors
/// The type of errors that can happen in the application.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error emitted during the start of an application in an execution engine.
    #[error("Could not start the application")]
    ApplicationCannotStart {
        // The original error.
        source: Box<dyn std::error::Error>,
    },

    /// Error emitted during the stop of an application in an execution engine.
    #[error("Could not stop the application")]
    ApplicationCannotStop {
        // The original error.
        source: Box<dyn std::error::Error>,
    },

    /// Unable to get the running state of an application in an execution engine.
    #[error("Could not get the state of the application")]
    ApplicationStateUnavailable {
        // The original error.
        source: Box<dyn std::error::Error>,
    },
}

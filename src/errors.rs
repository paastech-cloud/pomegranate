use std::boxed::Box;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not start the application")]
    ApplicationCannotStart { source: Box<dyn std::error::Error> },
    #[error("Could not stop the application")]
    ApplicationCannotStop { source: Box<dyn std::error::Error> },
    #[error("Could not get the state of the application")]
    ApplicationStateUnavailable { source: Box<dyn std::error::Error> },
}

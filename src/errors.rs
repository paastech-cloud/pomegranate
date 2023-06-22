use std::boxed::Box;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ApplicationStartError {
    pub source: Box<dyn Error>,
}

impl fmt::Display for ApplicationStartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The application could not start: {}", self.source)
    }
}

impl Error for ApplicationStartError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

#[derive(Debug)]
pub struct ApplicationStopError {
    pub source: Box<dyn Error>,
}

impl fmt::Display for ApplicationStopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The application could not be stopped: {}", self.source)
    }
}

impl Error for ApplicationStopError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

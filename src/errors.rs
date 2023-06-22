use std::boxed::Box;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ApplicationStartError {
    pub source: Box<dyn Error>,
}

impl fmt::Display for ApplicationStartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "The application could not start: {}",
            self.source.to_string()
        )
    }
}

impl Error for ApplicationStartError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

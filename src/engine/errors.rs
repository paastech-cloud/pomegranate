use std::{error::Error, fmt::Display};

use tonic::codegen::http::StatusCode;

#[derive(Debug)]
pub struct EngineError {
    error: anyhow::Error,
    pub code: StatusCode,
}

impl Error for EngineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.error.source()
    }

    #[allow(deprecated)]
    fn description(&self) -> &str {
        self.error.description()
    }
}

impl Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

impl From<anyhow::Error> for EngineError {
    fn from(error: anyhow::Error) -> Self {
        match error.downcast::<bollard::errors::Error>() {
            Ok(err) => err.into(),
            Err(err) => EngineError {
                error: err,
                code: StatusCode::INTERNAL_SERVER_ERROR,
            },
        }
    }
}

impl From<bollard::errors::Error> for EngineError {
    fn from(err: bollard::errors::Error) -> Self {
        translate_err(err)
    }
}

fn translate_err(err: bollard::errors::Error) -> EngineError {
    match err {
        bollard::errors::Error::DockerResponseServerError {
            status_code,
            message: _,
        } => EngineError {
            code: StatusCode::from_u16(status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            error: anyhow::Error::new(err),
        },
        _ => EngineError {
            error: anyhow::Error::new(err),
            code: StatusCode::INTERNAL_SERVER_ERROR,
        },
    }
}

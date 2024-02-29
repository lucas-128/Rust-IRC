use std::error::Error;
use std::fmt::Display;
use std::sync::{MutexGuard, PoisonError};

#[derive(Debug)]
pub struct ServerError {
    pub msg: String,
}
impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for ServerError {
    fn description(&self) -> &str {
        &self.msg
    }
}
impl From<ServerError> for String {
    fn from(error: ServerError) -> Self {
        error.to_string()
    }
}

impl ServerError {
    pub fn new(msg: &str) -> ServerError {
        ServerError {
            msg: msg.to_string(),
        }
    }
}

impl<R> From<PoisonError<MutexGuard<'_, R>>> for ServerError {
    fn from(err: PoisonError<MutexGuard<'_, R>>) -> ServerError {
        ServerError::new(&format!("Ocurrio error al utilizar lock: {}", err))
    }
}

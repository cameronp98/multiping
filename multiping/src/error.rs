use std::fmt;
use std::io;
use std::sync::mpsc::RecvError;

use crate::server::RemoteClientId;

/// The `Result` subtype for this crate
pub type Result<T> = std::result::Result<T, Error>;

/// The enum of all error values for this crate
#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    JsonError(serde_json::Error),
    RecvError(RecvError),
    SendError,
    JoinError,
    InvalidRemoteClientId(RemoteClientId),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IoError(e) => e.fmt(f),
            Error::JsonError(e) => e.fmt(f),
            Error::RecvError(e) => e.fmt(f),
            Error::SendError => write!(f, "SendError"),
            Error::JoinError => write!(f, "JoinError"),
            Error::InvalidRemoteClientId(id) => {
                f.debug_tuple("InvalidRemoteClientId").field(id).finish()
            }
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JsonError(err)
    }
}

impl From<RecvError> for Error {
    fn from(err: RecvError) -> Error {
        Error::RecvError(err)
    }
}

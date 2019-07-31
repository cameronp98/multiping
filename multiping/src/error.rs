use std::fmt;
use std::io;

use crate::connection::ConnectionId;
use crate::Message;

/// The `Result` subtype for this crate
pub type Result<T> = std::result::Result<T, Error>;

/// The enum of all error values for this crate
#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    JsonError(serde_json::Error),
    SenderDisconnected,
    ReceiverDisconnected,
    SendError,
    ThreadJoinError,
    InvalidConnectionId(ConnectionId),
    MutexLockError,
    UnexpectedMessage(Message),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IoError(e) => e.fmt(f),
            Error::JsonError(e) => e.fmt(f),
            Error::SenderDisconnected => write!(f, "recv failed: sender disconnected"),
            Error::SendError => write!(f, "failed to send on a channel"),
            Error::ThreadJoinError => write!(f, "failed to join a thread"),
            Error::MutexLockError => write!(f, "failed to lock mutex"),
            Error::InvalidConnectionId(id) => write!(f, "invalid client id {}", id),
            Error::ReceiverDisconnected => write!(f, "recv failed: receiver disconnected"),
            Error::UnexpectedMessage(msg) => write!(f, "unexpected message: {}", msg),
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

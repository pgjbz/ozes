use std::io::{Error as IOError, ErrorKind};
use std::{error::Error, fmt::Display};

pub type OzResult<T> = std::result::Result<T, OzesError>;

#[derive(Debug)]
pub enum OzesError {
    TimeOut,
    WithouConnection,
    ErrorResponse,
    UnknownError,
    ToLongMessage,
    AddrInUse,
    PermissionDenied,
}

impl Display for OzesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            Self::ErrorResponse => "response with error",
            Self::TimeOut => "to long wait",
            Self::WithouConnection => "lose connection",
            Self::UnknownError => "unknown error",
            Self::ToLongMessage => "to long message",
            Self::AddrInUse => "address already in use",
            Self::PermissionDenied => "permission denied",
        };
        write!(f, "{}", error)
    }
}

impl Error for OzesError {}

impl From<IOError> for OzesError {
    fn from(e: IOError) -> Self {
        match e.kind() {
            ErrorKind::ConnectionReset | ErrorKind::BrokenPipe => Self::WithouConnection,
            ErrorKind::TimedOut => Self::TimeOut,
            ErrorKind::AddrInUse => Self::AddrInUse,
            ErrorKind::PermissionDenied => Self::PermissionDenied,
            _ => OzesError::UnknownError,
        }
    }
}

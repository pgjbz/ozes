use std::io::{Error as IOError, ErrorKind};
use std::{error::Error, fmt::Display};

pub type OzResult<T> = std::result::Result<T, OzesError>;

#[derive(Debug)]
pub enum OzesError {
    TimeOut,
    WithouConnection,
    ErrorResponse,
    UnknownError(String),
    ToLongMessage,
    AddrInUse,
    PermissionDenied,
}

impl OzesError {
    pub fn is_error(&self, error: OzesError) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(&error)
    }
}

impl Display for OzesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            Self::ErrorResponse => "response with error".to_owned(),
            Self::TimeOut => "to long wait".to_owned(),
            Self::WithouConnection => "lose connection".to_owned(),
            Self::ToLongMessage => "to long message".to_owned(),
            Self::AddrInUse => "address already in use".to_owned(),
            Self::PermissionDenied => "permission denied".to_owned(),
            Self::UnknownError(error) => format!("unknown error: {}", error),
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
            _ => OzesError::UnknownError(e.to_string()),
        }
    }
}

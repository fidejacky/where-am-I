use std::fmt;

// Strings, not ureq's error type, so the HTTP library stays out of the public API.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Network(String),
    BadResponse(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Network(msg) => write!(f, "network error: {msg}"),
            Error::BadResponse(msg) => write!(f, "unusable response: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

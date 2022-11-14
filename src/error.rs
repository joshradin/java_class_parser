//! Contains the error type that can be emitted

use nom::error::ParseError;
use nom::Needed;
use std::backtrace::Backtrace;
use std::fmt::{Debug, Display, Formatter};
use std::io;

/// The error type
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    backtrace: Backtrace,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{} at {}", self.kind, self.backtrace)
        } else {
            write!(f, "{}", self.kind)
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    /// Gets the error kind
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl<E: Into<ErrorKind>> From<E> for Error {
    fn from(error: E) -> Self {
        let kind = error.into();
        Self {
            kind,
            backtrace: Backtrace::capture(),
        }
    }
}

/// The error kind
#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("{0} is not a known constant pool tag")]
    UnknownConstantPoolInfoTag(u8),
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error("Missing {:?} bytes", 0)]
    MissingBytes(Needed),
    #[error(transparent)]
    NomError {
        kind: nom::Err<nom::error::Error<Vec<u8>>>,
    },
}

impl<'a> From<nom::Err<nom::error::Error<&'a [u8]>>> for ErrorKind {
    fn from(e: nom::Err<nom::error::Error<&'a [u8]>>) -> Self {
        Self::NomError { kind: e.to_owned() }
    }
}

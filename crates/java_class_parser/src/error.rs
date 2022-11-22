//! Contains the error type that can be emitted

use crate::FQNameBuf;
use nom::Needed;
use std::backtrace::Backtrace;
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::path::PathBuf;

/// The error type
pub struct Error {
    kind: ErrorKind,
    backtrace: Backtrace,
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#}", self)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{} at\n{}", self.kind, self.backtrace)
        } else {
            write!(f, "{}", self.kind)
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    /// Create a new error from an an error kind
    pub fn new<E: Into<ErrorKind>>(kind: E) -> Self {
        Self {
            kind: kind.into(),
            backtrace: Backtrace::capture(),
        }
    }
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
    /// No class could be found for a given path
    #[error("No class found for path {0:?}")]
    NoClassFound(FQNameBuf),
    /// Encountered an unsupported classpath entry
    #[error("Unsupported entry in classpath: {0:?}")]
    UnsupportedEntry(PathBuf),
    /// An unknown tag was found in the constant pool
    #[error("{0} is not a known constant pool tag")]
    UnknownConstantPoolInfoTag(u8),
    /// An io error occurred
    #[error(transparent)]
    IoError(#[from] io::Error),
    /// While parsing, some bytes were missing
    #[error("Missing {:?} bytes", 0)]
    MissingBytes(Needed),
    /// A nom error occurred
    #[error(transparent)]
    NomError {
        /// the nom error kind
        kind: nom::Err<nom::error::Error<Vec<u8>>>,
    },
    /// A zip error occurred.
    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),
    /// Adding inheritance failed
    #[error("adding inheritance of {0} failed")]
    AddingInheritanceFailed(FQNameBuf),
}

impl<'a> From<nom::Err<nom::error::Error<&'a [u8]>>> for ErrorKind {
    fn from(e: nom::Err<nom::error::Error<&'a [u8]>>) -> Self {
        Self::NomError { kind: e.to_owned() }
    }
}

use self::Error::*;
use std::fmt::{Display, Formatter};

/// This is a library-specific error that is returned by all calls to all APIs.
#[derive(Debug)]
pub enum Error {
    /// The library could not be opened.
    OpeningLibraryError(String),
    /// The symbol could not be found.
    SymbolNotFound(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OpeningLibraryError(msg) => write!(f, "Could not open library: {}", msg),
            SymbolNotFound(symbol) => write!(f, "Symbol `{}` not found", symbol),
        }
    }
}

impl std::error::Error for Error {}

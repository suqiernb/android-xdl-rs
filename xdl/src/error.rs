use self::Error::*;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    OpeningLibraryError(String),
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

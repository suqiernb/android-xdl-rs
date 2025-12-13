use crate::error::Error;
use crate::raw::Library;

mod container;

pub use container::Container;

pub trait Symbols: Sized {
    unsafe fn load_from(lib: &Library) -> Result<Self, Error>;
}

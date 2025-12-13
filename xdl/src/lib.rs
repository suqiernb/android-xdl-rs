#![cfg(target_os = "android")]
#![allow(unused_imports, dead_code)]

mod error;
mod mut_ptr;
mod ptr;
pub mod raw;
mod symbol;
#[cfg(feature = "wrapper")]
pub mod wrapper;
#[cfg(feature = "derive")]
pub use android_xdl_derive as derive;

pub use error::Error;
pub use mut_ptr::RowPtrMut;
pub use ptr::RowPtr;
pub use symbol::{Library, Symbol};

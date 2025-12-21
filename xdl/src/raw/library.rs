use super::api::*;
use crate::{Error, mut_ptr::RowPtrMut, ptr::RowPtr};
use std::ffi::CStr;
use std::mem::{size_of, transmute_copy};
use std::os::raw::c_void;

pub type Handle = *mut c_void;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Library {
    handle: Handle,
}

impl Library {
    pub unsafe fn new(handle: Handle) -> Result<Self> {
        let handle: Option<_> = handle.into();
        handle.map(|handle| Self { handle }).ok_or_else(|| {
            Error::OpeningLibraryError("Cannot create `Library` from null pointer.".to_string())
        })
    }

    pub unsafe fn open(name: &CStr) -> Result<Self> {
        unsafe { Self::open_with_flags(name, XDL_DEFAULT) }
    }

    pub unsafe fn open_with_flags(name: &CStr, flags: i32) -> Result<Self> {
        unsafe {
            if !name.is_empty() {
                Self::new(xdl_open(name.as_ptr(), flags)).map_err(|_| {
                    Error::OpeningLibraryError(format!("`{}`", name.to_string_lossy()))
                })
            } else {
                Err(Error::OpeningLibraryError(
                    "The library name must not be empty.".to_string(),
                ))
            }
        }
    }

    pub unsafe fn symbol<T: Sized>(
        &self,
        name: &CStr,
        symbol_size: Option<&mut usize>,
    ) -> Result<T> {
        unsafe {
            assert_type_size::<T>();
            let size_ptr = match symbol_size {
                Some(size) => size,
                None => std::ptr::null_mut(),
            };
            let symbol: Option<_> = xdl_sym(self.handle, name.as_ptr(), size_ptr).into();
            symbol
                .map(|symbol| transmute_copy(&symbol))
                .ok_or_else(|| Error::SymbolNotFound(name.to_string_lossy().to_string()))
        }
    }

    pub unsafe fn debug_symbol<T: Sized>(
        &self,
        name: &CStr,
        symbol_size: Option<&mut usize>,
    ) -> Result<T> {
        unsafe {
            assert_type_size::<T>();
            let size_ptr = match symbol_size {
                Some(size) => size,
                None => std::ptr::null_mut(),
            };
            let symbol: Option<_> = xdl_dsym(self.handle, name.as_ptr(), size_ptr).into();
            symbol
                .map(|symbol| transmute_copy(&symbol))
                .ok_or_else(|| Error::SymbolNotFound(name.to_string_lossy().to_string()))
        }
    }

    pub unsafe fn handle(&self) -> Handle {
        self.handle
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe {
            let handle = xdl_close(self.handle);
            if !handle.is_null() {
                libc::dlclose(handle);
            }
        }
    }
}

unsafe impl Send for Library {}
unsafe impl Sync for Library {}

#[inline]
fn assert_type_size<T>() {
    if size_of::<T>() != size_of::<*mut ()>() {
        panic!(
            "The type passed to xdl::Library::symbol() function has a different size than a \
                 pointer - cannot transmute"
        );
    }
}

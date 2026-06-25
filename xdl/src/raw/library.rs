use super::api::*;
use crate::Error;
use std::ffi::CStr;
use std::mem::{size_of, transmute_copy};
use std::os::raw::c_void;
use std::ptr::NonNull;

type Result<T> = std::result::Result<T, Error>;

/**
Main interface for opening and working with a dynamic link library.

**Note:** The handle to the library gets released when the library object gets dropped.
Unless your application opened the library multiple times, this is the moment when symbols
obtained from the library become dangling symbols.
*/
#[derive(Debug)]
pub struct Library {
    handle: Handle,
}

impl Library {
    /// Create Library from Library handle.
    pub unsafe fn new(handle: Handle) -> Result<Self> {
        let handle = NonNull::new(handle);
        handle
            .map(|handle| Self {
                handle: handle.as_ptr(),
            })
            .ok_or_else(|| {
                Error::OpeningLibraryError("Cannot create `Library` from null pointer.".to_string())
            })
    }

    /// Open dynamic library using provided file name or path.
    pub unsafe fn open(name: &CStr) -> Result<Self> {
        unsafe { Self::open_with_flags(name, XDL_DEFAULT) }
    }

    /// Open a dynamic library with flags.
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

    /// Obtains a symbol from the opened library.
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
            let symbol = NonNull::new(xdl_sym(self.handle, name.as_ptr(), size_ptr));
            symbol
                .map(|symbol| transmute_copy(&symbol.as_ptr()))
                .ok_or_else(|| Error::SymbolNotFound(name.to_string_lossy().to_string()))
        }
    }

    /// Obtains a debug symbol from the opened library.
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
            let symbol = NonNull::new(xdl_dsym(self.handle, name.as_ptr(), size_ptr));
            symbol
                .map(|symbol| transmute_copy(&symbol.as_ptr()))
                .ok_or_else(|| Error::SymbolNotFound(name.to_string_lossy().to_string()))
        }
    }

    /// Returns the raw handle for the opened library.
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
const fn assert_type_size<T: Sized>() {
    const {
        assert!(
            size_of::<T>() == size_of::<*mut ()>(),
            "Cannot transmute: type size must match pointer size for safe transmutation."
        )
    };
}

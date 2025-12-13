use crate::mut_ptr::RowPtrMut;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A wrapper around C pointers that provides convenient conversion to `Option<*const T>`.
///
/// This type is designed to simplify working with C pointers in Rust FFI code,
/// particularly when you need to convert between raw pointers and optional pointers
/// for safe handling of potentially null or invalid pointers.
///
/// # Design Intent
///
/// The primary purpose of `RowPtr` is to provide an ergonomic way to convert
/// `*const T` into `Option<*const T>` using Rust's `Into` trait.
/// This is especially useful when working with C libraries that may return
/// null pointers to indicate errors.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use std::os::raw::c_void;
/// use xdl::ptr::RowPtr;
///
/// // Create from a raw pointer
/// let raw_ptr: *const c_void = 0x1000 as *const _;
///
/// // Convert to Option<*const c_void>
/// let opt_ptr: Option<*const c_void> = raw_ptr.into();
/// assert!(opt_ptr.is_some());
///
/// // Handle null pointers
/// let null_ptr = RowPtr::new::<c_void>(std::ptr::null());
/// let opt_null: Option<*const c_void> = null_ptr.into();
/// assert!(opt_null.is_none());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct RowPtr<'a, T> {
    ptr: *const T,
    phantom: PhantomData<&'a T>,
}

impl<'a, T> RowPtr<'a, T> {
    /// Creates a new `RowPtr` from a raw `*const T` pointer.
    ///
    /// This constructor doesn't perform any validation on the pointer.
    /// The validation happens when converting to `Option<*const T>`.
    pub fn new(ptr: *const T) -> Self {
        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Deref for RowPtr<'a, T> {
    type Target = *const T;
    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}

impl<'a, T> From<*const T> for RowPtr<'a, T> {
    /// Converts a raw `*const T` pointer into a `RowPtr`.
    ///
    /// This conversion is infallible and always succeeds.
    fn from(ptr: *const T) -> Self {
        Self::new(ptr)
    }
}

impl<'a, T> From<RowPtr<'a, T>> for Option<*const T> {
    /// Converts a `RowPtr` into an `Option<*const T>`.
    ///
    /// This is the primary conversion that `RowPtr` is designed to provide.
    /// It filters out null pointers.
    fn from(ptr: RowPtr<'_, T>) -> Self {
        if ptr.is_null() { None } else { Some(*ptr) }
    }
}

unsafe impl<'a, T> Send for RowPtr<'a, T> {}
unsafe impl<'a, T> Sync for RowPtr<'a, T> {}

impl<'a, T> From<RowPtrMut<'a, T>> for RowPtr<'a, T> {
    /// Converts a mutable pointer wrapper into an immutable pointer wrapper.
    ///
    /// This is a safe conversion because it only reduces the pointer's capabilities.
    /// Converting from mutable to immutable never violates memory safety.
    fn from(ptr: RowPtrMut<'a, T>) -> Self {
        Self::new(*ptr as *const T)
    }
}

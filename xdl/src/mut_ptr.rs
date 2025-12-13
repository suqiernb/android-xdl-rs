use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A wrapper around C pointers that provides convenient conversion to `Option<*mut T>`.
///
/// This type is designed to simplify working with C pointers in Rust FFI code,
/// particularly when you need to convert between raw pointers and optional pointers
/// for safe handling of potentially null or invalid pointers.
///
/// # Design Intent
///
/// The primary purpose of `RowPtrMut` is to provide an ergonomic way to convert
/// `*mut T` into `Option<*mut T>` using Rust's `Into` trait.
/// This is especially useful when working with C libraries that may return
/// null pointers to indicate errors.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use std::os::raw::c_void;
/// use xdl::ptr::RowPtrMut;
///
/// // Create from a raw pointer
/// let raw_ptr: *mut c_void = 0x1000 as *mut _;
///
/// // Convert to Option<*mut c_void>
/// let opt_ptr: Option<*mut c_void> = raw_ptr.into();
/// assert!(opt_ptr.is_some());
///
/// // Handle null pointers
/// let null_ptr = RowPtr::new::<c_void>(std::ptr::null_mut());
/// let opt_null: Option<*mut c_void> = null_ptr.into();
/// assert!(opt_null.is_none());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct RowPtrMut<'a, T> {
    ptr: *mut T,
    phantom: PhantomData<&'a T>,
}

impl<'a, T> RowPtrMut<'a, T> {
    /// Creates a new `RowPtrMut` from a raw `*mut T` pointer.
    ///
    /// This constructor doesn't perform any validation on the pointer.
    /// The validation happens when converting to `Option<*mut T>`.
    pub fn new(ptr: *mut T) -> Self {
        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Deref for RowPtrMut<'a, T> {
    type Target = *mut T;
    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}

impl<'a, T> From<*mut T> for RowPtrMut<'a, T> {
    /// Converts a raw `*mut T` pointer into a `RowPtrMut`.
    ///
    /// This conversion is infallible and always succeeds.
    fn from(ptr: *mut T) -> Self {
        Self::new(ptr)
    }
}

impl<'a, T> From<RowPtrMut<'a, T>> for Option<*mut T> {
    /// Converts a `RowPtrMut` into an `Option<*mut T>`.
    ///
    /// This is the primary conversion that `RowPtrMut` is designed to provide.
    /// It filters out null pointers.
    fn from(ptr: RowPtrMut<'_, T>) -> Self {
        if ptr.is_null() { None } else { Some(ptr.ptr) }
    }
}

unsafe impl<'a, T> Send for RowPtrMut<'a, T> {}
unsafe impl<'a, T> Sync for RowPtrMut<'a, T> {}

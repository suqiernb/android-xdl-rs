mod library;
pub use library::*;

use std::marker::PhantomData;
use std::mem::transmute_copy;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_void;

#[derive(Debug, Clone, Copy)]
pub struct Symbol<'lib, T: 'lib> {
    symbol: T,
    _phantom: PhantomData<&'lib T>,
}

impl<'lib, T> Symbol<'lib, T> {
    pub fn new(symbol: T) -> Self {
        Self {
            symbol,
            _phantom: PhantomData,
        }
    }

    pub fn from_row(ptr: *const c_void) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self::new(unsafe { transmute_copy(&ptr) }))
        }
    }
}

impl<'lib, T> Deref for Symbol<'lib, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.symbol
    }
}

impl<'lib, T> DerefMut for Symbol<'lib, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.symbol
    }
}

unsafe impl<'lib, T: Send> Send for Symbol<'lib, T> {}
unsafe impl<'lib, T: Sync> Sync for Symbol<'lib, T> {}

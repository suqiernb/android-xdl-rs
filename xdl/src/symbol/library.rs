use crate::Error;
use crate::raw::Library as RowLibrary;
use crate::symbol::Symbol;
use std::ffi::CStr;
use std::ops::Deref;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Library {
    lib: RowLibrary,
}

impl Library {
    pub fn open<S: AsRef<CStr>>(name: S) -> Result<Self> {
        unsafe { RowLibrary::open(name.as_ref()).map(Self::from) }
    }

    pub fn open_with_flags<S: AsRef<CStr>>(name: S, flags: i32) -> Result<Self> {
        unsafe { RowLibrary::open_with_flags(name.as_ref(), flags).map(Self::from) }
    }

    pub fn symbol<T>(&self, name: &CStr) -> Result<Symbol<'_, T>> {
        unsafe { self.lib.symbol(name, None).map(Symbol::new) }
    }

    pub fn symbol_and_size<T>(&self, name: &CStr) -> Result<(Symbol<'_, T>, usize)> {
        unsafe {
            let mut symbol_size: usize = 0;
            self.lib
                .symbol(name, Some(&mut symbol_size))
                .map(|sym| (Symbol::new(sym), symbol_size))
        }
    }

    pub fn debug_symbol<T>(&self, name: &CStr) -> Result<Symbol<'_, T>> {
        unsafe { self.lib.debug_symbol(name, None).map(Symbol::new) }
    }

    pub fn debug_symbol_and_size<T>(&self, name: &CStr) -> Result<(Symbol<'_, T>, usize)> {
        unsafe {
            let mut symbol_size: usize = 0;
            self.lib
                .debug_symbol(name, Some(&mut symbol_size))
                .map(|sym| (Symbol::new(sym), symbol_size))
        }
    }
}

impl Deref for Library {
    type Target = RowLibrary;

    fn deref(&self) -> &Self::Target {
        &self.lib
    }
}

impl From<RowLibrary> for Library {
    fn from(value: RowLibrary) -> Self {
        Self { lib: value }
    }
}

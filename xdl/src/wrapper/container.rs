use super::Symbols;
use crate::Library;
use crate::error::Error;
use std::ops::{Deref, DerefMut};

pub struct Container<T: Symbols> {
    lib: Library,
    api: T,
}

impl<T: Symbols> Container<T> {
    pub fn from(lib: Library) -> Result<Self, Error> {
        let api = unsafe { T::load_from(&lib)? };
        Ok(Self { lib, api })
    }

    pub fn library(&self) -> &Library {
        &self.lib
    }
}

impl<T: Symbols> Deref for Container<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.api
    }
}

impl<T: Symbols> DerefMut for Container<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.api
    }
}

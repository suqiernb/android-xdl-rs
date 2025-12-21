use super::Symbols;
use crate::Library;
use crate::error::Error;
use std::ops::{Deref, DerefMut};

/**
Container for both a dynamically loaded library handle and its API.

Keeping both library and its symbols together makes it safe to use it because symbols are released
together with the library. `Container` also doesn't have any external lifetimes - this makes it
easy to use `Container` inside structures.

#Example

```no_run
use android_xdl::Library;
use android_xdl::wrapper::Container;
use android_xdl::derive::NativeBridge;
use std::os::raw::c_char;
use std::ffi::CStr;

#[derive(NativeBridge)]
struct Example<'a> {
    do_something: extern "C" fn(),
    add_one: unsafe extern "C" fn (arg: i32) -> i32,
    global_count: &'a mut u32,
    c_string: *const c_char,
}

// wrapper for c_string won't be generated, implement it here
impl<'a> Example<'a> {
    #[inline]
    pub fn c_string(&self) -> &CStr {
        unsafe {CStr::from_ptr(self.c_string)}
    }
}

fn main () {
    let mut api: Container<Example> = Container::from(
        Library::open("libexample.so").unwrap()
    ).unwrap();

    api.do_something();
    let _result = unsafe { api.add_one(5) };
    *api.global_count_mut() += 1;
    println!("C string: {}", api.c_string().to_str().unwrap())
}
```
*/
pub struct Container<T: Symbols> {
    lib: Library,
    api: T,
}

impl<T: Symbols> Container<T> {
    /// Load all symbols from the library.
    pub fn from(lib: Library) -> Result<Self, Error> {
        let api = unsafe { T::load_from(&lib)? };
        Ok(Self { lib, api })
    }

    /// Returns a reference to the underlying [`Library`] handle.
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

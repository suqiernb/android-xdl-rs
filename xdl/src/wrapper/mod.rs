/*!
High-level and safe API for opening and getting symbols from dynamic link libraries.
It is based on wrapping private symbols with public functions to prevent direct access
and supports automatic loading of symbols into structures.

This API solves the problem with dangling symbols by wrapping raw symbols with public functions.
User of API does not have direct access to raw symbols and therefore symbols cannot be copied.
Symbols and library handle are kept in one `Container` structure and therefore both the the library
and symbols get released at the same time.

# Example

```no_run
use android_xdl::Library;
use android_xdl::wrapper::Container;
use android_xdl::derive::NativeBridge;

#[derive(NativeBridge)]
struct Example<'a> {
    do_something: extern "C" fn(),
    add_one: unsafe extern "C" fn (arg: i32) -> i32,
    global_count: &'a mut u32,
}

fn main () {
    let mut api: Container<Example> = Container::from(
        Library::open("libexample.so").unwrap()
    ).unwrap();

    api.do_something();
    let _result = unsafe { api.add_one(5) };
    *api.global_count_mut() += 1;

    //symbols are released together with library handle
    //this prevents dangling symbols
    drop(api);
}
```

Unfortunately in Rust it is not possible to create an API for dynamic link libraries that would
be 100% safe. This API aims to be 99% safe by providing zero cost functional wrappers around
raw symbols. However, it is possible to make a mistake if you create API as a standalone object
(not in container):

# Example of a mistake - dangling symbol

```no_run
use android_xdl::Library;
use android_xdl::wrapper::Container;
use android_xdl::derive::NativeBridge;

#[derive(NativeBridge)]
struct Example<'a> {
    do_something: extern "C" fn(),
    add_one: unsafe extern "C" fn (arg: i32) -> i32,
    global_count: &'a mut u32,
}

fn main () {
    let lib = Library::open("libexample.so").unwrap();
    let mut api = unsafe{Example::load_from(&lib)};
    drop(lib);

    //api has now dangling symbols
}
```

To prevent this mistake don't use structures implementing `Symbols` directly, but rather use
`Container` as in the first example.
*/

use crate::error::Error;
use crate::raw::Library;

mod container;

pub use container::Container;

/**
Trait for binding library API.

This trait is intended to be used with `#[derive(NativeBridge)]` macro defined in the
`android_xdl_derive` crate. It forces several restrictions on types that implement it:

* Only structures can implement this trait.
* All fields need to be private.
* Only functions, references and pointers are allowed.
* You can't define a type using `type Fun =fn();` and use it in the structure. This is a limitation
  of the Rust reflection mechanism. Only raw functions, references and pointers are allowed.

The `derive` macro not only generates implementation of `load_from()` function, but it also generates
safe wrappers around the loaded symbols. These wrappers are named exactly like the field that
they wrap.
Wrappers are not generated only for:
* Pointers - there is no safe way of preventing dangling symbols if a user has a direct access to
  pointers. The recommended approach here is to either use references instead of pointers or
  to manually create safe wrappers. For example C `const char *` can be manually converted into
  `& std::ffi::CStr`.
* Variadic functions. Rust doesn't have any mechanism that allows creating safe wrappers around
  them. You need to handle them manually.


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

**Note:** `NativeBridge` should only be used together with `Container` structure, never to create
a standalone object. API and library handle need to be kept together to prevent dangling symbols.

**Note:** By default obtained symbol name is the field name. You can change this by
assigning the `#[native(symbol = "...")]` attribute to the given field.

**Note:** By default `Error::SymbolNotFound` is returned if the loaded symbol name has a null value.
While null is a valid value of a exported symbol, it is usually not expected by users of libraries.
If a `null` value is acceptable for a pointer field in your scenario,
you should wrap the field's type in [`Option`].
*/
pub trait Symbols: Sized {
    unsafe fn load_from(lib: &Library) -> Result<Self, Error>;
}

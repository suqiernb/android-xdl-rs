/*!
Library for opening and working with dynamic link libraries (also known as shared object).
This is the FFI binding for [`xDL`](https://github.com/hexhacking/xDL)

# Quick example

```no_run
use android_xdl::Library;
use android_xdl::wrapper::Container;
use android_xdl::derive::NativeBridge;

#[derive(NativeBridge)]
struct Example<'a> {
    example_fun: fn(arg: i32) -> u32,
    example_unsafe_fun: unsafe extern "C" fn(),
    example_reference: &'a mut i32,
    // A function or field may not always exist in the library.
    example_unsafe_fun_option: Option<unsafe extern "C" fn()>,
    example_reference_option: Option<&'a mut i32>,
}

fn main() {
    let mut api: Container<Example> = Container::from(
        Library::open("libexample.so").unwrap()
    ).unwrap();

    api.example_fun(5);
    unsafe { api.example_unsafe_fun() };
    *api.example_reference_mut() = 5;

    // Optional functions return Some(result) if the function is present or None if absent.
    unsafe { api.example_unsafe_fun_option() };
    // Optional fields are Some(value) if present and None if absent.
    if let Some(example_reference) = api.example_reference_option() {
        *example_reference = 5;
    }
}
```

# Features

* Bypass the restrictions of Android 7.0+ linker namespace.
* Lookup dynamic link symbols in `.dynsym`.
* Lookup debuging symbols in `.symtab` and "`.symtab` in `.gnu_debugdata`".
* Support Android 4.1 - 16 (API level 16 - 36).
* Support armeabi-v7a, arm64-v8a, x86 and x86_64.

# API Design & Features

* Has a low-level API that provides full flexibility of using libraries.
* Has two high-level APIs that protect against dangling symbols - each in its own way.
* High level APIs support automatic loading of symbols into structures. You only need to define a
  structure that represents an API. The rest happens automatically and requires only minimal amount of code.
* Automatic loading of symbols helps you to follow the DRY paradigm.

# Documentation

[Cargo documentation](https://docs.rs/android_xdl)

[Examples](https://github.com/suqiernb/android-xdl-rs/master/example)

# License
This code is licensed under the [MIT](https://github.com/suqiernb/android-xdl-rs/master/LICENSE) license.

# Acknowledgement

[xDL](https://github.com/hexhacking/xDL): makes all these possible.

# Comparison of APIs:

* [**raw**](./raw/index.html) - a low-level API. It is mainly intended to give you full flexibility
  if you decide to create you own custom solution for handling dynamic link libraries.
  For typical operations you probably should use one of high-level APIs.

* [**symbol**](./symbol/struct.Symbol.html) - a high-level API. It prevents dangling symbols by creating
  zero cost structural wrappers around symbols obtained from the library. These wrappers use
  Rust borrowing mechanism to make sure that the library will never get released before obtained
  symbols.

* [**wrapper**](./wrapper/index.html) - a high-level API. It prevents dangling symbols by creating
  zero cost functional wrappers around symbols obtained from the library. These wrappers prevent
  accidental copying of raw symbols from library API. Dangling symbols are prevented by keeping
  library and its API in one structure - this makes sure that symbols and library are released
  together.
*/

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

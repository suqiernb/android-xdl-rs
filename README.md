# xDL

![](https://img.shields.io/badge/license-MIT-brightgreen.svg?style=flat)
![](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat)
![](https://img.shields.io/badge/Android-4.1%20--%2016-blue.svg?style=flat)
![](https://img.shields.io/badge/arch-armeabi--v7a%20%7C%20arm64--v8a%20%7C%20x86%20%7C%20x86__64-blue.svg?style=flat)

xDL is an enhanced implementation of the Android DL series functions.

[**简体中文**](https://github.com/suqiernb/android-xdl-rs/blob/master/README.zh-CN.md)

> [!WARNING]
> Currently in a preliminary state of availability, the api may be unstable.

## Features

* Enhanced `dlopen()` + `dlsym()` + `dladdr()`.
    * Bypass the restrictions of Android 7.0+ linker namespace.
    * Lookup dynamic link symbols in `.dynsym`.
    * Lookup debuging symbols in `.symtab` and "`.symtab` in `.gnu_debugdata`".
* Enhanced `dl_iterate_phdr()`.
    * Compatible with Android 4.x on ARM32.
    * Including linker / linker64 (for Android <= 8.x).
    * Return full pathname instead of basename (for Android 5.x).
    * Return app\_process32 / app\_process64 instead of package name.
* Support Android 4.1 - 16 (API level 16 - 36).
* Support armeabi-v7a, arm64-v8a, x86 and x86_64.


## How to use
> this library is [xDL](https://github.com/hexhacking/xDL) rust binding, provides a safe and easy to use API, dynamic link library for the Android platform on the loading and symbol lookup.

### Install
```toml
[dependencies]
android_xdl = { version = "0.0.2", features = ["derive"] }
```

### Manually loading symbols
```rust
use std::os::raw::*;
use android_xdl::{Library, Error};


#[allow(non_camel_case_types)]
type fn_puts_t = unsafe extern "C" fn(*const c_char) -> c_int;

fn main() -> Result<(), Error> {
    let library = Library::open(c"libc.so")?;

    let symbol = library.symbol::<fn_puts_t>(c"puts")?;
    
    let string = c">> Hello World !\n>> 中文字符测试\n>> 表情符号测试😎";
    unsafe { symbol(string.as_ptr()) };

    Ok(())
}
```

### Use derive macros
```rust
use std::os::raw::*;
use android_xdl::wrapper::Container;
use android_xdl::{Error, Library};
use android_xdl::derive::NativeBridge;

#[derive(NativeBridge)]
struct LibcApi {
    puts: unsafe extern "C" fn(*const c_char) -> c_int,
    getpid: unsafe extern "C" fn() -> c_int,
    getuid: unsafe extern "C" fn() -> c_uint,
}

fn main() -> Result<(), Error> {
    let api = Container::<LibcApi>::from(Library::open(c"libc.so")?)?;

    unsafe {
        let pid = api.getpid();
        let uid = api.getuid();
        log::debug!("PID: {}, UID: {}", pid, uid);

        api.puts(c"puts: \tHello World\n\t中文字符测试\n\t表情符号测试😎".as_ptr());
    }

    Ok(())
}
```

## Credits

* [xDL](https://github.com/hexhacking/xDL): fork source

## License

MIT licensed, as found in the [LICENSE](https://github.com/suqiernb/android-xdl-rs/blob/master/LICENSE) file.

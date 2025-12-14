# xDL

![](https://img.shields.io/badge/license-MIT-brightgreen.svg?style=flat)
![](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat)
![](https://img.shields.io/badge/Android-4.1%20--%2016-blue.svg?style=flat)
![](https://img.shields.io/badge/arch-armeabi--v7a%20%7C%20arm64--v8a%20%7C%20x86%20%7C%20x86__64-blue.svg?style=flat)

xDL 是 Android DL 系列函数的增强实现。

[**English**](https://github.com/suqiernb/android-xdl-rs/blob/master/README.md)

> [!WARNING]
> 目前处于初步可用状态, api 可能不稳定.

## 特性

* 增强的 `dlopen()` + `dlsym()` + `dladdr()`。
    * 绕过 Android 7.0+ linker namespace 的限制。
    * 查询 `.dynsym` 中的动态链接符号。
    * 查询 `.symtab` 和 “`.gnu_debugdata` 里的 `.symtab`” 中的调试符号。
* 增强的 `dl_iterate_phdr()`。
    * 兼容 ARM32 平台的 Android 4.x。
    * 在 Android <= 8.x 时，包含 linker / linker64。
    * 在 Android 5.x 中，返回完整的路径名（full pathname），而不是文件名（basename）。
    * 返回 app\_process32 / app\_process64，而不是包名。
* 支持 Android 4.1 - 16 (API level 16 - 36)。
* 支持 armeabi-v7a, arm64-v8a, x86 和 x86_64。


## 快速指南
> 这个库是 [xDL](https://github.com/hexhacking/xDL) 的 rust 绑定, 提供了一套安全且易用的 API，用于在 Android 平台上进行动态链接库的加载和符号查找。

### 添加依赖
```toml
[dependencies]
android_xdl = { version = "0.0.2", features = ["derive"] }
```

### 手动加载
```rust
use std::os::raw::*;
use android_xdl::{Library, Error};


#[allow(non_camel_case_types)]
type fn_puts_t = unsafe extern "C" fn(*const c_char) -> c_int;

fn main() -> Result<(), Error> {
    // 加载库
    let library = Library::open(c"libc.so")?;
    // 获取 puts 函数指针
    let symbol = library.symbol::<fn_puts_t>(c"puts")?;
    // 调用函数
    let string = c">> Hello World !\n>> 中文字符测试\n>> 表情符号测试😎";
    unsafe { symbol(string.as_ptr()) };

    Ok(())
}
```

### 使用派生宏
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
        // 测试 PID 和 UID
        let pid = api.getpid();
        let uid = api.getuid();
        log::debug!("PID: {}, UID: {}", pid, uid);
        // 测试 puts
        api.puts(c"puts: \tHello World\n\t中文字符测试\n\t表情符号测试😎".as_ptr());
    }

    Ok(())
}
```

## 致谢

* [xDL](https://github.com/hexhacking/xDL): 这一切的源头

## 许可证

使用 [MIT 许可证](LICENSE)

use std::error::Error;
use std::os::raw::*;
use android_xdl::Library;

#[allow(non_camel_case_types)]
type fn_puts_t = unsafe extern "C" fn(*const c_char) -> c_int;

fn main() -> Result<(), Box<dyn Error>> {
    let library = Library::open(c"libc.so")?;
    println!("Successfully opened libc, handle: {:p}", unsafe {
        library.handle()
    });
    let symbol = library.symbol::<fn_puts_t>(c"puts")?;

    println!("Found 'puts' symbol at address: {:p}", *symbol);
    let string = c">> Hello World !\n>> 中文字符测试\n>> 表情符号测试😎";
    unsafe { symbol(string.as_ptr()) };

    Ok(())
}

use chrono::Local;
use env_logger::fmt::style::Style;
use std::ffi::CString;
use std::io::Write;
use std::os::raw::*;
use android_xdl::wrapper::Container;
use android_xdl::{Error, Library};
use android_xdl_derive::NativeBridge;

type Result<T> = std::result::Result<T, Error>;

#[derive(NativeBridge)]
#[native(logger)]
struct LibcApi {
    puts: unsafe extern "C" fn(*const c_char) -> c_int,
    getpid: unsafe extern "C" fn() -> c_int,
    getuid: unsafe extern "C" fn() -> c_uint,
    strlen: unsafe extern "C" fn(*const c_char) -> usize,
    strncpy: unsafe extern "C" fn(dst: *mut c_char, src: *const c_char, n: usize) -> *mut c_char,
}

fn example() -> Result<()> {
    let api = Container::<LibcApi>::from(Library::open(c"libc.so")?)?;

    unsafe {
        // 测试 PID 和 UID
        let pid = api.getpid();
        let uid = api.getuid();
        log::debug!("PID: {}, UID: {}", pid, uid);

        // 测试 puts
        api.puts(c"puts: \tHello World\n\t中文字符测试\n\t表情符号测试😎".as_ptr());

        // 测试字符串拷贝
        let str1 = CString::from(c"Hello World\n");
        let str1_len = api.strlen(str1.as_ptr());
        log::trace!("strlen: {}", str1_len);
        let mut buff: Vec<c_char> = Vec::with_capacity(128);
        api.strncpy(buff.as_mut_ptr(), str1.as_ptr(), str1_len);
        buff[str1_len] = b'\0';
        api.puts(buff.as_ptr());
    }

    Ok(())
}

fn main() {
    init_logger();
    if let Err(e) = example() {
        log::error!("{}", e);
    }
}

fn init_logger() {
    let env = env_logger::Env::default().default_filter_or("trace");
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            let level = record.level();
            let style = buf.default_level_style(level).bold();
            let dimmed = Style::new().dimmed();

            writeln!(
                buf,
                "{} {style}{level:05}{style:#} {dimmed}[ {} ]{dimmed:#} {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.module_path().unwrap_or("<unnamed>"),
                record.args()
            )
        })
        .init();
}

use android_xdl::derive::NativeBridge;
use android_xdl::wrapper::Container;
use android_xdl::{Error, Library};
use chrono::Local;
use env_logger::fmt::style::Style;
use std::io::Write;
use std::os::raw::*;

type Result<T> = std::result::Result<T, Error>;

#[derive(NativeBridge)]
#[native(logger)]
struct LibcApi {
    puts: unsafe extern "C" fn(*const c_char) -> c_int,
    getpid: unsafe extern "C" fn() -> c_int,
    getuid: unsafe extern "C" fn() -> c_uint,
    #[native(symbol = "this_function_does_not_exist")]
    non_existent_function: Option<unsafe extern "C" fn() -> c_int>,
}

fn example() -> Result<()> {
    let api = Container::<LibcApi>::from(Library::open(c"libc.so")?)?;

    unsafe {
        api.puts(c"puts: \tHello World\n\t中文字符测试\n\t表情符号测试😎".as_ptr());

        let pid = api.getpid();
        let uid = api.getuid();
        log::debug!("PID: {pid}, UID: {uid}");

        if api.has_non_existent_function() {
            log::error!("`int this_function_does_not_exist()` exists unexpectedly!");
        }
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

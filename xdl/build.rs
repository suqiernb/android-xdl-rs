use std::path::PathBuf;

const SOURCE_PATH: &str = "c/xdl/src/main/cpp";

fn main() {
    let target = std::env::var("TARGET").unwrap();
    if !target.contains("android") {
        panic!("Only 'android' target is supported, found: {}", target);
    }

    cc::Build::new()
        .include(format!("{}/include", SOURCE_PATH))
        .include(SOURCE_PATH)
        .files(get_sources())
        .flag("-std=c17")
        .flags(["-Wall", "-Os", "-ffunction-sections", "-fdata-sections"])
        .compile("xdl");

    println!("cargo:rustc-link-arg=-Wl,--exclude-libs,ALL");
    println!("cargo:rustc-link-arg=-Wl,--gc-sections");
    if target.contains("aarch64") || target.contains("x86_64") {
        println!("cargo:rustc-link-arg=-Wl,-z,max-page-size=16384");
    }
}

fn get_sources() -> Vec<PathBuf> {
    let mut sources = Vec::new();
    if let Ok(entries) = std::fs::read_dir(SOURCE_PATH) {
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().map(|e| e == "c").unwrap_or(false) {
                sources.push(path);
            }
        }
    }
    sources
}

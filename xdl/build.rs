use std::path::Path;

fn main() {
    if std::env::var("DOCS_RS").is_ok() {
        return;
    }

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os != "android" {
        println!("cargo:warning=`xdl` is designed for Android targets");
        return;
    }

    let xdl_source = Path::new("clib/xdl/src/main/cpp");
    cc::Build::new()
        .include(xdl_source.join("include"))
        .file(xdl_source.join("xdl.c"))
        .file(xdl_source.join("xdl_iterate.c"))
        .file(xdl_source.join("xdl_linker.c"))
        .file(xdl_source.join("xdl_lzma.c"))
        .file(xdl_source.join("xdl_util.c"))
        .flag("-std=c17")
        .flags(["-Wall", "-Os", "-ffunction-sections", "-fdata-sections"])
        .compile("xdl");
}

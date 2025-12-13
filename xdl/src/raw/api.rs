pub use libc::dl_phdr_info;
use libc::{Elf32_Phdr, Elf64_Phdr};
use std::os::raw::*;

#[link(name = "xdl")]
unsafe extern "C" {
    // Enhanced dlopen() / dlclose() / dlsym().
    pub fn xdl_open(filename: *const c_char, flags: c_int) -> *mut c_void;
    pub fn xdl_open2(info: *mut dl_phdr_info) -> *mut c_void;
    pub fn xdl_close(handle: *mut c_void) -> *mut c_void;
    pub fn xdl_sym(
        handle: *mut c_void,
        symbol: *const c_char,
        symbol_size: *mut usize,
    ) -> *mut c_void;
    pub fn xdl_dsym(
        handle: *mut c_void,
        symbol: *const c_char,
        symbol_size: *mut usize,
    ) -> *mut c_void;
    // Enhanced dladdr().
    pub fn xdl_addr(addr: *mut c_void, info: *mut xdl_info_t, cache: *mut *mut c_void) -> c_int;
    pub fn xdl_addr4(
        addr: *mut c_void,
        info: *mut xdl_info_t,
        cache: *mut *mut c_void,
        flags: c_int,
    ) -> c_int;
    pub fn xdl_addr_clean(cache: *mut *mut c_void);
    // Enhanced dl_iterate_phdr().
    pub fn xdl_iterate_phdr(
        callback: unsafe extern "C" fn(
            info: *mut dl_phdr_info,
            size: usize,
            data: *mut c_void,
        ) -> c_int,
        data: *mut c_void,
        flags: c_int,
    ) -> c_int;
    // Custom dlinfo().
    pub fn xdl_info(handle: *mut c_void, request: c_int, info: *mut c_void) -> c_int;
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct xdl_info_t {
    /// Pathname of shared object that contains address.
    pub dli_fname: *const c_char,
    /// Address at which shared object is loaded.
    pub dli_fbase: *mut c_void,
    /// Name of nearest symbol with address lower than addr.
    pub dli_sname: *const c_char,
    /// Exact address of symbol named in dli_sname.
    pub dli_saddr: *mut c_void,
    /// Symbol size of nearest symbol with address lower than addr.
    pub dli_ssize: usize,
    /// Pointer to array of ELF program headers for this object.
    #[cfg(target_pointer_width = "64")]
    pub dlpi_phdr: *const Elf64_Phdr,
    #[cfg(target_pointer_width = "32")]
    pub dlpi_phdr: *const Elf32_Phdr,
    /// Number of items in dlpi_phdr.
    pub dlpi_phnum: usize,
}

// Default value for flags in `xdl_open`, `xdl_addr4`, `and xdl_iterate_phdr`.
pub const XDL_DEFAULT: i32 = 0x00;

// xdl_open / xdl_close / xdl_sym.
pub const XDL_TRY_FORCE_LOAD: i32 = 0x01;
pub const XDL_ALWAYS_FORCE_LOAD: i32 = 0x02;

// xdl_addr
pub const XDL_NON_SYM: i32 = 0x01;

// xdl_iterate_phdr
pub const XDL_FULL_PATHNAME: i32 = 0x01;

// xdl_info
pub const XDL_DI_DLINFO: i32 = 0x01;

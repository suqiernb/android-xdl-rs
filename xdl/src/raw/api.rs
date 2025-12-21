/*!
This module provides safe(r) FFI bindings to the [`xDL`](https://github.com/hexhacking/xDL) C library,
enabling bypassing linker namespace restrictions and advanced symbol lookup on Android.

# Safety
Most functions in this module are `unsafe` due to the nature of raw pointer and dynamic linking operations.
Callers must ensure that pointers are valid and handles are used correctly.
*/
#![allow(non_camel_case_types)]
pub use libc::dl_phdr_info;
use libc::{Elf32_Phdr, Elf64_Phdr};
use std::os::raw::*;

/// A handle to a dynamically loaded library.
pub type Handle = *mut c_void;

/// Default loading behavior for [`xdl_open`].
///
/// If the library has already been loaded into memory (e.g., by the system linker),
/// [`xdl_open`] will not call `dlopen` again. It will simply return a valid handle
/// to the existing loaded library.
pub const XDL_DEFAULT: c_int = 0x00;

/// Try to force load the library if not already loaded.
///
/// If the library is not currently loaded in memory, [`xdl_open`] will attempt to
/// load it using `dlopen`. If it's already loaded, behaves like [`XDL_DEFAULT`].
///
/// This is useful when you want to ensure a library is available but don't want
/// to reload it unnecessarily.
pub const XDL_TRY_FORCE_LOAD: c_int = 0x01;

/// Always force load the library, even if already loaded.
///
/// [`xdl_open`] will always call `dlopen` to load the library, regardless of
/// whether it's already loaded in memory. This may result in multiple copies
/// of the same library being loaded at different addresses.
///
/// Use this flag when you need a fresh instance of the library, separate from
/// any existing instances.
pub const XDL_ALWAYS_FORCE_LOAD: c_int = 0x02;

/// Skip symbol lookup, only get ELF information.
///
/// When this flag is set, [`xdl_addr4`] will not attempt to find the nearest symbol
/// for the given address. The `dli_sname`, `dli_saddr`, and `dli_ssize` fields
/// in the returned [`xdl_info_t`] will all be set to zero/null.
///
/// This can improve performance when you only need to know which ELF contains
/// an address, but don't need specific symbol information.
pub const XDL_NON_SYM: c_int = 0x01;

/// Always return full pathnames instead of basenames.
///
/// When this flag is set, [`xdl_iterate_phdr`] will provide complete filesystem paths
/// for each shared object in the `dlpi_name` field of [`dl_phdr_info`].
pub const XDL_FULL_PATHNAME: c_int = 0x01;

/// Request ELF information structure.
///
/// When passed as the `request` parameter to [`xdl_info`], this causes the function
/// to populate an [`xdl_info_t`] structure with information about the library handle.
///
/// Note: The symbol-related fields (`dli_sname`, `dli_saddr`, `dli_ssize`) in the
/// returned structure will be zero/null, as this request only provides library-level
/// information, not symbol-level information.
pub const XDL_DI_DLINFO: c_int = 0x01;

/// Extended information returned by [`xdl_addr`] and [`xdl_info`].
#[repr(C)]
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct xdl_info_t {
    /// Pathname of shared object that contains address.
    pub dli_fname: *const c_char,
    /// Address at which shared object is loaded.
    pub dli_fbase: *mut c_void,
    /// Name of nearest symbol with address lower than addr.
    pub dli_sname: *const c_char,
    /// Exact address of symbol named in `dli_sname`.
    pub dli_saddr: *mut c_void,
    /// Symbol size of nearest symbol with address lower than addr.
    pub dli_ssize: usize,
    /// Pointer to array of ELF program headers for this object.
    #[cfg(target_pointer_width = "64")]
    pub dlpi_phdr: *const Elf64_Phdr,
    #[cfg(target_pointer_width = "32")]
    pub dlpi_phdr: *const Elf32_Phdr,
    /// Number of items in `dlpi_phdr`.
    pub dlpi_phnum: usize,
}

/// Callback type for [`xdl_iterate_phdr`].
pub type xdl_iterate_phdr_callback_t =
    unsafe extern "C" fn(info: *mut dl_phdr_info, size: usize, data: *mut c_void) -> c_int;

#[link(name = "xdl")]
unsafe extern "C" {
    /// Opens a shared library, similar to [`dlopen`](https://man7.org/linux/man-pages/man3/dlopen.3.html).
    ///
    /// This function can bypass the restrictions of Android 7.0+ linker namespace.
    ///
    /// Depending on the `flags` parameter, the behavior will differ:
    /// - [`XDL_DEFAULT`]: If the library is already loaded, `xdl_open()` will not `dlopen()` it again
    ///   (but still returns a valid handle).
    /// - [`XDL_TRY_FORCE_LOAD`]: If the library is not loaded, try to `dlopen()` it.
    /// - [`XDL_ALWAYS_FORCE_LOAD`]: Always `dlopen()` the library.
    ///
    /// If `xdl_open()` actually uses `dlopen()` to load the library, [`xdl_close()`] will return
    /// the handle from the linker (the return value of `dlopen()`). Otherwise, it returns `null`.
    ///
    /// `filename` can be a basename or full pathname. On Android 7.0+, if passing a basename,
    /// ensure no duplicate ELF is loaded in the current process, as `xdl_open()` returns only
    /// the first matching ELF.
    pub fn xdl_open(filename: *const c_char, flags: c_int) -> Handle;

    /// Creates a handle from an existing [`dl_phdr_info`] structure.
    ///
    /// This function always uses [`XDL_DEFAULT`] semantics and will not attempt to load
    /// the ELF with `dlopen()`.
    ///
    /// # Safety
    /// `info` must point to a valid `dl_phdr_info` structure.
    pub fn xdl_open2(info: *mut dl_phdr_info) -> Handle;

    /// Closes a library handle, similar to [`dlclose()`](https://man7.org/linux/man-pages/man3/dlclose.3.html).
    ///
    /// **Important behavior**: The return value indicates whether the underlying library
    /// needs to be closed with standard `dlclose()`.
    ///
    /// - **Returns the original linker handle** (non-null): If [`xdl_open()`] actually used
    ///   `dlopen()` to load the library (i.e., when called with [`XDL_TRY_FORCE_LOAD`] or
    ///   [`XDL_ALWAYS_FORCE_LOAD`] flags and the library wasn't already loaded).
    ///   In this case, you **should** call `dlclose()` on the returned handle when appropriate.
    ///
    /// - **Returns `null`**: If [`xdl_open()`] did not invoke `dlopen()` (i.e., [`XDL_DEFAULT`]
    ///   flag with an already-loaded library, or [`xdl_open2()`]). No further cleanup is needed.
    ///
    /// This design allows you to manage the actual `dlopen()`/`dlclose()` lifecycle separately
    /// from xDL's handle management.
    ///
    /// # Safety
    /// `handle` must be a valid handle obtained from [`xdl_open`] or [`xdl_open2`].
    pub fn xdl_close(handle: Handle) -> Handle;

    /// Looks up a dynamic symbol in the library's `.dynsym` section.
    ///
    /// Similar to [`dlsym()`](https://man7.org/linux/man-pages/man3/dlsym.3.html), but only searches
    /// dynamic linking symbols. If `symbol_size` is not `null`, it will be set to the size (in bytes)
    /// of the symbol's content in the ELF.
    pub fn xdl_sym(handle: Handle, symbol: *const c_char, symbol_size: *mut usize) -> *mut c_void;

    /// Looks up a debugging symbol in the library's `.symtab` section.
    ///
    /// Searches debugging symbols in `.symtab` and `.gnu_debugdata`. This is slower than [`xdl_sym()`]
    /// as it reads from disk. Note that `.dynsym` and `.symtab` symbol sets do not fully overlap.
    pub fn xdl_dsym(handle: Handle, symbol: *const c_char, symbol_size: *mut usize) -> *mut c_void;

    /// Enhanced version of [`dladdr()`](https://man7.org/linux/man-pages/man3/dladdr.3.html).
    ///
    /// Can lookup both dynamic and debugging symbols. Uses the [`xdl_info_t`] structure which
    /// contains more information than standard `Dl_info`.
    ///
    /// The `cache` parameter caches the ELF handle for faster subsequent calls to the same ELF.
    /// Use [`xdl_addr_clean()`] to clear the cache when done.
    pub fn xdl_addr(addr: *mut c_void, info: *mut xdl_info_t, cache: *mut *mut c_void) -> c_int;

    /// Enhanced `dladdr()` with additional flags.
    ///
    /// When `flags` is [`XDL_DEFAULT`], behaves identically to [`xdl_addr()`].
    /// When `flags` is [`XDL_NON_SYM`], skips symbol lookup (sets `dli_sname`, `dli_saddr`,
    /// `dli_ssize` to zero/null).
    pub fn xdl_addr4(
        addr: *mut c_void,
        info: *mut xdl_info_t,
        cache: *mut *mut c_void,
        flags: c_int,
    ) -> c_int;

    /// Cleans up the cache created by [`xdl_addr()`] or [`xdl_addr4()`].
    ///
    /// # Safety
    /// `cache` must be a valid pointer obtained from [`xdl_addr`] or [`xdl_addr4`].
    pub fn xdl_addr_clean(cache: *mut *mut c_void);

    /// Enhanced version of [`dl_iterate_phdr()`](https://man7.org/linux/man-pages/man3/dl_iterate_phdr.3.html).
    ///
    /// Compatible with Android 4.x on ARM32, and always includes linker/linker64.
    ///
    /// The `flags` parameter can be [`XDL_DEFAULT`] or [`XDL_FULL_PATHNAME`] (to return full paths
    /// instead of basenames).
    pub fn xdl_iterate_phdr(
        callback: xdl_iterate_phdr_callback_t,
        data: *mut c_void,
        flags: c_int,
    ) -> c_int;

    /// Custom implementation similar to [`dlinfo()`](https://man7.org/linux/man-pages/man3/dlinfo.3.html).
    ///
    /// Currently only supports [`XDL_DI_DLINFO`] request, which populates an [`xdl_info_t`] structure
    /// (note: symbol fields will be zero/null).
    ///
    /// Returns `0` on success, `-1` on failure.
    ///
    /// # Safety
    /// `handle` must be valid and `info` must point to writable memory of appropriate size.
    pub fn xdl_info(handle: Handle, request: c_int, info: *mut c_void) -> c_int;
}

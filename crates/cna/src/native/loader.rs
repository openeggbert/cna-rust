//! Platform dynamic-library ownership and symbol resolution.

use core::ffi::{c_char, c_int, c_void};
use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};

use crate::error::{CnaError, Result};

#[cfg(unix)]
const RTLD_NOW: c_int = 2;

#[cfg(all(unix, not(target_os = "macos")))]
#[link(name = "dl")]
extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
    fn dlerror() -> *const c_char;
}

#[cfg(target_os = "macos")]
extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
    fn dlerror() -> *const c_char;
}

#[cfg(unix)]
#[derive(Debug)]
pub(super) struct Library(*mut c_void);

#[cfg(unix)]
impl Drop for Library {
    fn drop(&mut self) {
        // SAFETY: `self.0` is a successful `dlopen` result and Arc ownership
        // keeps it live until every copied CNA function pointer is unreachable.
        let _ = unsafe { dlclose(self.0) };
    }
}

#[cfg(unix)]
unsafe impl Send for Library {}
#[cfg(unix)]
unsafe impl Sync for Library {}

#[cfg(unix)]
impl Library {
    pub(super) fn open(path: &Path) -> core::result::Result<Self, String> {
        let text = path.as_os_str().to_string_lossy();
        let name = CString::new(text.as_bytes()).map_err(|_| "path contains NUL".to_owned())?;
        // SAFETY: `name` is NUL-terminated, lives through the call, and RTLD_NOW
        // requests immediate resolution without transferring Rust-owned memory.
        let handle = unsafe { dlopen(name.as_ptr(), RTLD_NOW) };
        if handle.is_null() {
            Err(loader_error())
        } else {
            Ok(Self(handle))
        }
    }

    pub(super) unsafe fn symbol<T: Copy>(&self, name: &'static str) -> Result<T> {
        let symbol_name = CString::new(name).expect("static symbol names contain no NUL");
        // SAFETY: `self.0` is live and `symbol_name` is a valid C string.
        let pointer = unsafe { dlsym(self.0, symbol_name.as_ptr()) };
        if pointer.is_null() {
            return Err(CnaError::MissingSymbol(name));
        }
        debug_assert_eq!(core::mem::size_of::<T>(), core::mem::size_of_val(&pointer));
        // SAFETY: callers choose the exact audited function-pointer type for
        // this named symbol; all supported targets represent it in one pointer.
        Ok(unsafe { core::mem::transmute_copy(&pointer) })
    }
}

#[cfg(unix)]
fn loader_error() -> String {
    // SAFETY: `dlerror` returns either null or a library-owned NUL-terminated
    // string that remains valid until the next dynamic-loader call on this thread.
    let pointer = unsafe { dlerror() };
    if pointer.is_null() {
        "unknown dynamic-loader error".to_owned()
    } else {
        // SAFETY: non-null `dlerror` results satisfy the C string contract.
        unsafe { CStr::from_ptr(pointer) }
            .to_string_lossy()
            .into_owned()
    }
}

pub(super) fn library_candidates() -> Vec<PathBuf> {
    let filename = platform_library_name();
    if let Some(path) = std::env::var_os("CNA_NATIVE_LIBRARY") {
        return vec![PathBuf::from(path)];
    }

    let mut candidates = Vec::new();
    if let Some(directory) = std::env::var_os("CNA_NATIVE_DIR") {
        candidates.push(PathBuf::from(directory).join(filename));
    }
    if let Some(root) = std::env::var_os("CNA_ROOT") {
        let root = PathBuf::from(root);
        candidates.push(root.join("build/modules/c-api").join(filename));
        candidates.push(root.join("cmake-build-debug/modules/c-api").join(filename));
    }
    if let Ok(executable) = std::env::current_exe() {
        if let Some(directory) = executable.parent() {
            candidates.push(directory.join(filename));
        }
    }
    candidates.push(PathBuf::from(filename));
    candidates
}

#[cfg(target_os = "linux")]
const fn platform_library_name() -> &'static str {
    "libcna_c_api.so"
}

#[cfg(target_os = "macos")]
const fn platform_library_name() -> &'static str {
    "libcna_c_api.dylib"
}

#[cfg(target_os = "windows")]
const fn platform_library_name() -> &'static str {
    "cna_c_api.dll"
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
const fn platform_library_name() -> &'static str {
    "cna_c_api"
}

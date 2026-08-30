//! Platform dynamic-library ownership and symbol resolution.

use core::ffi::{c_char, c_int, c_void};
use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};

use crate::error::{CnaError, Result};

/// NUL-terminates a wide path for a Win32 `W` entry point.
///
/// Compiled on every host, not only Windows, so its tests run everywhere.
///
/// Kept platform-independent so the part most likely to be wrong -- an interior
/// NUL slipping through, or a missing terminator -- is unit-tested on every
/// host rather than only where the loader itself can be compiled.
#[cfg_attr(not(windows), allow(dead_code))]
fn terminated_wide(units: impl IntoIterator<Item = u16>) -> core::result::Result<Vec<u16>, String> {
    let mut wide: Vec<u16> = units.into_iter().collect();
    if wide.contains(&0) {
        return Err("path contains NUL".to_owned());
    }
    wide.push(0);
    Ok(wide)
}

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

#[cfg(windows)]
mod windows {
    use core::ffi::c_void;
    use std::ffi::CString;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    use crate::error::{CnaError, Result};

    use super::terminated_wide;

    // Win32 uses the `system` calling convention, which differs from `C` on
    // 32-bit Windows. Naming it explicitly keeps the declaration correct on
    // every Windows architecture rather than only on x86-64.
    #[link(name = "kernel32")]
    extern "system" {
        fn LoadLibraryW(name: *const u16) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const i8) -> *mut c_void;
        fn FreeLibrary(module: *mut c_void) -> i32;
        fn GetLastError() -> u32;
    }

    #[derive(Debug)]
    pub(in crate::native) struct Library(*mut c_void);

    impl Drop for Library {
        fn drop(&mut self) {
            // SAFETY: `self.0` is a successful `LoadLibraryW` result and Arc
            // ownership keeps it live until every copied CNA function pointer
            // is unreachable.
            let _ = unsafe { FreeLibrary(self.0) };
        }
    }

    // SAFETY: a Windows module handle is process-global and the loader is
    // internally synchronized, exactly as the Unix branch documents.
    unsafe impl Send for Library {}
    unsafe impl Sync for Library {}

    impl Library {
        pub(in crate::native) fn open(path: &Path) -> core::result::Result<Self, String> {
            // `encode_wide` is exact: a Windows path is already UTF-16, and a
            // lossy conversion through `str` would corrupt one containing an
            // unpaired surrogate.
            let wide = terminated_wide(path.as_os_str().encode_wide())?;
            // SAFETY: `wide` is NUL-terminated and lives through the call.
            let handle = unsafe { LoadLibraryW(wide.as_ptr()) };
            if handle.is_null() {
                // SAFETY: `GetLastError` reads this thread's last error code.
                Err(format!("LoadLibraryW failed with error {}", unsafe {
                    GetLastError()
                }))
            } else {
                Ok(Self(handle))
            }
        }

        pub(in crate::native) unsafe fn symbol<T: Copy>(&self, name: &'static str) -> Result<T> {
            let symbol_name = CString::new(name).expect("static symbol names contain no NUL");
            // SAFETY: `self.0` is live and `symbol_name` is a valid C string.
            let pointer = unsafe { GetProcAddress(self.0, symbol_name.as_ptr()) };
            if pointer.is_null() {
                return Err(CnaError::MissingSymbol(name));
            }
            debug_assert_eq!(core::mem::size_of::<T>(), core::mem::size_of_val(&pointer));
            // SAFETY: callers choose the exact audited function-pointer type
            // for this named symbol; all supported targets represent it in one
            // pointer.
            Ok(unsafe { core::mem::transmute_copy(&pointer) })
        }
    }
}

#[cfg(windows)]
pub(super) use windows::Library;

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

#[cfg(test)]
mod tests {
    use super::terminated_wide;

    #[test]
    fn a_wide_path_is_nul_terminated_exactly_once() {
        let wide = terminated_wide("C:\\cna.dll".encode_utf16()).expect("no interior NUL");
        assert_eq!(wide.last(), Some(&0));
        assert_eq!(wide.iter().filter(|unit| **unit == 0).count(), 1);
        assert_eq!(
            String::from_utf16(&wide[..wide.len() - 1]).expect("round-trips"),
            "C:\\cna.dll"
        );
    }

    #[test]
    fn an_interior_nul_is_refused_rather_than_truncating_the_path() {
        assert!(terminated_wide("C:\\cna\u{0}.dll".encode_utf16()).is_err());
    }

    #[test]
    fn a_lone_surrogate_survives_the_conversion() {
        // A Windows path may hold an unpaired surrogate. It must reach
        // LoadLibraryW unchanged, which is why the loader encodes the OsStr
        // directly instead of going through a lossy `str`.
        let units = [0xD800_u16, 0x0041];
        let wide = terminated_wide(units).expect("no interior NUL");
        assert_eq!(wide, vec![0xD800, 0x0041, 0]);
    }
}

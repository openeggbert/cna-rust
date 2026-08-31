//! Where the audited function tables come from.
//!
//! There is one route inventory and one safe layer above it, and two ways to
//! fill the tables underneath: the platform's dynamic loader, or CNA's symbols
//! resolved at link time. Which one a build uses is a Cargo feature; nothing
//! above this module can tell the difference, and neither can the ABI gate,
//! which checks both.

#[cfg(feature = "dynamic-loading")]
use core::ffi::{c_char, c_int, c_void};
#[cfg(feature = "dynamic-loading")]
use std::ffi::{CStr, CString};
#[cfg(feature = "dynamic-loading")]
use std::path::{Path, PathBuf};

use crate::error::{CnaError, Result};

/// Where one build's function pointers come from.
///
/// Dynamic mode owns a `Library` whose lifetime keeps every resolved pointer
/// valid. Direct mode owns nothing: the symbols are part of the executable, so
/// there is no handle to hold and nothing to unload. Representing that as a
/// variant rather than as an `Option<Library>` keeps the difference stated
/// instead of encoded in a null.
#[derive(Debug)]
pub(crate) enum NativeSource {
    /// Resolved through the platform loader; the library must outlive the table.
    #[cfg(all(feature = "dynamic-loading", any(unix, windows)))]
    Dynamic(Library),
    /// Resolved at link time. There is no library object, and inventing one
    /// would be a lie about what can be unloaded.
    #[cfg(feature = "direct-link")]
    Linked,
}

/// Fills one table field from whichever source this build uses.
///
/// The name is an identifier, not a string, so that direct mode can reach the
/// declaration of that exact route: `sys::linked::$name` carries the route's
/// real parameter and return types, and assigning it to the field's `_fn`
/// alias is a coercion the compiler checks. A signature that drifts from the
/// canonical header therefore fails to compile rather than being transmuted
/// into place. Dynamic mode resolves the same name as a string, and
/// `tools/native-abi/verify.py` proves the field, the name and the alias agree
/// in both modes.
macro_rules! acquire {
    ($source:expr, $name:ident, $ty:ty) => {{
        #[cfg(feature = "direct-link")]
        {
            let _ = &$source;
            let route: $ty = cna_sys::linked::$name;
            route
        }
        #[cfg(not(feature = "direct-link"))]
        {
            // SAFETY: the requested type is this route's own canonical
            // function-pointer alias, which the ABI gate re-checks against the
            // header Clang parses.
            unsafe { $source.symbol::<$ty>(stringify!($name))? }
        }
    }};
}

pub(crate) use acquire;

impl NativeSource {
    /// Resolves one symbol through the platform loader.
    ///
    /// Only dynamic mode has anything to resolve. Direct mode never calls
    /// this: `acquire!` takes the linked declaration instead, so a build with
    /// no dynamic loader does not need one to exist.
    ///
    /// # Safety
    /// `T` must be the canonical function-pointer type declared for `name`.
    #[cfg_attr(feature = "direct-link", allow(dead_code))]
    pub(super) unsafe fn symbol<T: Copy>(&self, name: &'static str) -> Result<T> {
        match self {
            #[cfg(all(feature = "dynamic-loading", any(unix, windows)))]
            // SAFETY: the caller's obligation is carried straight through.
            Self::Dynamic(library) => unsafe { library.symbol::<T>(name) },
            #[cfg(feature = "direct-link")]
            Self::Linked => {
                let _ = name;
                Err(CnaError::UnsupportedRuntime(
                    "a directly linked build resolves no symbols at run time",
                ))
            }
        }
    }
}

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

// The dynamic loader below is compiled only when a build can use it. A
// directly linked build has no loader to call, and leaving the `dl*`
// declarations compiled would make the crate import symbols it never uses --
// which on a target with no dynamic loader at all is not merely untidy but
// unlinkable.

#[cfg(all(unix, feature = "dynamic-loading"))]
const RTLD_NOW: c_int = 2;

#[cfg(all(unix, not(target_os = "macos"), feature = "dynamic-loading"))]
#[link(name = "dl")]
extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
    fn dlerror() -> *const c_char;
}

#[cfg(all(target_os = "macos", feature = "dynamic-loading"))]
extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
    fn dlerror() -> *const c_char;
}

#[cfg(all(unix, feature = "dynamic-loading"))]
#[derive(Debug)]
pub(super) struct Library(*mut c_void);

#[cfg(all(unix, feature = "dynamic-loading"))]
impl Drop for Library {
    fn drop(&mut self) {
        // SAFETY: `self.0` is a successful `dlopen` result and Arc ownership
        // keeps it live until every copied CNA function pointer is unreachable.
        let _ = unsafe { dlclose(self.0) };
    }
}

#[cfg(all(unix, feature = "dynamic-loading"))]
unsafe impl Send for Library {}
#[cfg(all(unix, feature = "dynamic-loading"))]
unsafe impl Sync for Library {}

#[cfg(all(unix, feature = "dynamic-loading"))]
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

#[cfg(all(unix, feature = "dynamic-loading"))]
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

#[cfg(all(windows, feature = "dynamic-loading"))]
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

#[cfg(all(windows, feature = "dynamic-loading"))]
pub(super) use windows::Library;

#[cfg(feature = "dynamic-loading")]
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

#[cfg(all(target_os = "linux", feature = "dynamic-loading"))]
const fn platform_library_name() -> &'static str {
    "libcna_c_api.so"
}

#[cfg(all(target_os = "macos", feature = "dynamic-loading"))]
const fn platform_library_name() -> &'static str {
    "libcna_c_api.dylib"
}

#[cfg(all(target_os = "windows", feature = "dynamic-loading"))]
const fn platform_library_name() -> &'static str {
    "cna_c_api.dll"
}

#[cfg(all(
    feature = "dynamic-loading",
    not(any(target_os = "linux", target_os = "macos", target_os = "windows"))
))]
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

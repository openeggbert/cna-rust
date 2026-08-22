//! Audited dynamic C ABI boundary.

use core::ffi::{c_char, c_int, c_void};
use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use cna_sys as sys;

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
struct Library(*mut c_void);

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

#[derive(Debug)]
pub(crate) struct Native {
    #[cfg(unix)]
    _library: Library,
    error_get_last_message_size: sys::cna_error_get_last_message_size_fn,
    error_copy_last_message: sys::cna_error_copy_last_message_fn,
    game_create: sys::cna_game_create_fn,
    game_set_frame_hooks: sys::cna_game_set_frame_hooks_ext_fn,
    game_run: sys::cna_game_run_fn,
    game_request_exit: sys::cna_game_request_exit_fn,
    game_destroy: sys::cna_game_destroy_fn,
    game_get_graphics_device: sys::cna_game_get_graphics_device_fn,
    graphics_device_get_viewport: sys::cna_graphics_device_get_viewport_fn,
    graphics_device_clear_rgba: sys::cna_graphics_device_clear_rgba_fn,
    graphics_device_get_renderer_info: sys::cna_graphics_device_get_renderer_info_fn,
    graphics_device_get_renderer_name_size: sys::cna_graphics_device_get_renderer_name_size_fn,
    graphics_device_copy_renderer_name: sys::cna_graphics_device_copy_renderer_name_fn,
    texture2d_create_from_encoded_memory: sys::cna_texture2d_create_from_encoded_memory_fn,
    texture2d_get_info: sys::cna_texture2d_get_info_fn,
    texture2d_destroy: sys::cna_texture2d_destroy_fn,
    sprite_batch_create: sys::cna_sprite_batch_create_fn,
    sprite_batch_begin: sys::cna_sprite_batch_begin_fn,
    sprite_batch_submit_many: sys::cna_sprite_batch_submit_many_fn,
    sprite_batch_end: sys::cna_sprite_batch_end_fn,
    sprite_batch_destroy: sys::cna_sprite_batch_destroy_fn,
    keyboard_get_state: sys::cna_keyboard_get_state_fn,
}

impl Native {
    pub(crate) fn load() -> Result<Arc<Self>> {
        #[cfg(unix)]
        {
            Self::load_unix().map(Arc::new)
        }
        #[cfg(not(unix))]
        {
            Err(CnaError::UnsupportedPlatform)
        }
    }

    #[cfg(unix)]
    fn load_unix() -> Result<Self> {
        let candidates = library_candidates();
        let mut diagnostics = Vec::new();
        for candidate in &candidates {
            match Library::open(candidate) {
                Ok(library) => return Self::from_library(library),
                Err(error) => diagnostics.push(format!("{}: {error}", candidate.display())),
            }
        }
        Err(CnaError::NativeUnavailable {
            searched: candidates,
            details: diagnostics.join("; "),
        })
    }

    #[cfg(unix)]
    fn from_library(library: Library) -> Result<Self> {
        macro_rules! symbol {
            ($name:literal, $ty:ty) => {{
                // SAFETY: every requested type is copied directly from the
                // canonical CNA header declaration named by `$name`.
                unsafe { library.symbol::<$ty>($name)? }
            }};
        }

        let get_abi_version = symbol!("cna_get_abi_version", sys::cna_get_abi_version_fn);
        // SAFETY: the symbol has the audited zero-argument ABI declaration.
        let actual = unsafe { get_abi_version() };
        if actual != sys::CNA_ABI_VERSION {
            return Err(CnaError::AbiVersionMismatch {
                expected: sys::CNA_ABI_VERSION,
                actual,
            });
        }

        Ok(Self {
            error_get_last_message_size: symbol!(
                "cna_error_get_last_message_size",
                sys::cna_error_get_last_message_size_fn
            ),
            error_copy_last_message: symbol!(
                "cna_error_copy_last_message",
                sys::cna_error_copy_last_message_fn
            ),
            game_create: symbol!("cna_game_create", sys::cna_game_create_fn),
            game_set_frame_hooks: symbol!(
                "cna_game_set_frame_hooks_ext",
                sys::cna_game_set_frame_hooks_ext_fn
            ),
            game_run: symbol!("cna_game_run", sys::cna_game_run_fn),
            game_request_exit: symbol!("cna_game_request_exit", sys::cna_game_request_exit_fn),
            game_destroy: symbol!("cna_game_destroy", sys::cna_game_destroy_fn),
            game_get_graphics_device: symbol!(
                "cna_game_get_graphics_device",
                sys::cna_game_get_graphics_device_fn
            ),
            graphics_device_get_viewport: symbol!(
                "cna_graphics_device_get_viewport",
                sys::cna_graphics_device_get_viewport_fn
            ),
            graphics_device_clear_rgba: symbol!(
                "cna_graphics_device_clear_rgba",
                sys::cna_graphics_device_clear_rgba_fn
            ),
            graphics_device_get_renderer_info: symbol!(
                "cna_graphics_device_get_renderer_info",
                sys::cna_graphics_device_get_renderer_info_fn
            ),
            graphics_device_get_renderer_name_size: symbol!(
                "cna_graphics_device_get_renderer_name_size",
                sys::cna_graphics_device_get_renderer_name_size_fn
            ),
            graphics_device_copy_renderer_name: symbol!(
                "cna_graphics_device_copy_renderer_name",
                sys::cna_graphics_device_copy_renderer_name_fn
            ),
            texture2d_create_from_encoded_memory: symbol!(
                "cna_texture2d_create_from_encoded_memory",
                sys::cna_texture2d_create_from_encoded_memory_fn
            ),
            texture2d_get_info: symbol!("cna_texture2d_get_info", sys::cna_texture2d_get_info_fn),
            texture2d_destroy: symbol!("cna_texture2d_destroy", sys::cna_texture2d_destroy_fn),
            sprite_batch_create: symbol!(
                "cna_sprite_batch_create",
                sys::cna_sprite_batch_create_fn
            ),
            sprite_batch_begin: symbol!("cna_sprite_batch_begin", sys::cna_sprite_batch_begin_fn),
            sprite_batch_submit_many: symbol!(
                "cna_sprite_batch_submit_many",
                sys::cna_sprite_batch_submit_many_fn
            ),
            sprite_batch_end: symbol!("cna_sprite_batch_end", sys::cna_sprite_batch_end_fn),
            sprite_batch_destroy: symbol!(
                "cna_sprite_batch_destroy",
                sys::cna_sprite_batch_destroy_fn
            ),
            keyboard_get_state: symbol!("cna_keyboard_get_state", sys::cna_keyboard_get_state_fn),
            _library: library,
        })
    }

    pub(crate) fn check(&self, result: sys::CNA_Result) -> Result<()> {
        if result == sys::CNA_RESULT_SUCCESS {
            return Ok(());
        }
        Err(CnaError::Native {
            code: result,
            message: self.last_error_message(),
        })
    }

    pub(crate) fn create_game(
        &self,
        info: &sys::CNA_GameCreateInfo,
        handle: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: references provide initialized, live input/output objects for
        // the synchronous call; nested pointers are owned by the caller.
        self.check(unsafe { (self.game_create)(info, handle) })
    }

    pub(crate) fn set_game_frame_hooks(
        &self,
        game: sys::CNA_Handle,
        hooks: &sys::CNA_GameFrameHooks,
    ) -> Result<()> {
        // SAFETY: the internal caller supplies its live owned game handle and
        // CNA copies this fully initialized versioned structure.
        self.check(unsafe { (self.game_set_frame_hooks)(game, hooks) })
    }

    pub(crate) fn run_game(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: only the runner owns and uses this handle on its native thread.
        self.check(unsafe { (self.game_run)(game) })
    }

    pub(crate) fn request_game_exit(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: GameContext guarantees a live callback-scoped game handle.
        self.check(unsafe { (self.game_request_exit)(game) })
    }

    pub(crate) fn destroy_game(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: only the runner calls this for its exactly-once owned handle.
        self.check(unsafe { (self.game_destroy)(game) })
    }

    pub(crate) fn borrow_graphics_device(
        &self,
        game: sys::CNA_Handle,
        device: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the caller is callback-scoped and supplies a valid output.
        self.check(unsafe { (self.game_get_graphics_device)(game, device) })
    }

    pub(crate) fn clear_graphics_device(
        &self,
        device: sys::CNA_Handle,
        rgba: [f32; 4],
    ) -> Result<()> {
        // SAFETY: GraphicsDevice guarantees its callback-scoped handle.
        self.check(unsafe {
            (self.graphics_device_clear_rgba)(device, rgba[0], rgba[1], rgba[2], rgba[3])
        })
    }

    pub(crate) fn graphics_viewport(
        &self,
        device: sys::CNA_Handle,
        viewport: &mut sys::CNA_Viewport,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is initialized/live.
        self.check(unsafe { (self.graphics_device_get_viewport)(device, viewport) })
    }

    pub(crate) fn renderer_info(
        &self,
        device: sys::CNA_Handle,
        info: &mut sys::CNA_RendererInfo,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is initialized/live.
        self.check(unsafe { (self.graphics_device_get_renderer_info)(device, info) })
    }

    pub(crate) fn renderer_name_size(&self, device: sys::CNA_Handle, size: &mut u64) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is valid.
        self.check(unsafe { (self.graphics_device_get_renderer_name_size)(device, size) })
    }

    pub(crate) fn copy_renderer_name(
        &self,
        device: sys::CNA_Handle,
        destination: &mut [u8],
        copied: &mut u64,
    ) -> Result<()> {
        let capacity = u64::try_from(destination.len())
            .map_err(|_| CnaError::InvalidInput("renderer-name buffer is too large"))?;
        // SAFETY: the slice describes exactly `capacity` writable bytes and all
        // references remain live through the synchronous call.
        self.check(unsafe {
            (self.graphics_device_copy_renderer_name)(
                device,
                destination.as_mut_ptr().cast(),
                capacity,
                copied,
            )
        })
    }

    pub(crate) fn create_texture_from_encoded(
        &self,
        device: sys::CNA_Handle,
        bytes: &[u8],
        texture: &mut sys::CNA_Handle,
    ) -> Result<()> {
        let count = u64::try_from(bytes.len())
            .map_err(|_| CnaError::InvalidInput("encoded texture is too large"))?;
        // SAFETY: the encoded slice remains live and the null decode-info uses
        // CNA defaults; the output reference is valid.
        self.check(unsafe {
            (self.texture2d_create_from_encoded_memory)(
                device,
                bytes.as_ptr(),
                count,
                core::ptr::null(),
                texture,
            )
        })
    }

    pub(crate) fn texture_info(
        &self,
        texture: sys::CNA_Handle,
        info: &mut sys::CNA_Texture2DInfo,
    ) -> Result<()> {
        // SAFETY: the owned texture handle and initialized output are live.
        self.check(unsafe { (self.texture2d_get_info)(texture, info) })
    }

    pub(crate) fn destroy_texture(&self, texture: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the caller transfers exactly-once ownership of a live handle.
        self.check(unsafe { (self.texture2d_destroy)(texture) })
    }

    pub(crate) fn create_sprite_batch(
        &self,
        device: sys::CNA_Handle,
        batch: &mut sys::CNA_Handle,
    ) -> Result<()> {
        // SAFETY: the device is callback-scoped and output is valid.
        self.check(unsafe { (self.sprite_batch_create)(device, batch) })
    }

    pub(crate) fn begin_sprite_batch(
        &self,
        batch: sys::CNA_Handle,
        info: &sys::CNA_SpriteBatchBeginInfo,
    ) -> Result<()> {
        // SAFETY: the owned handle and versioned input are live.
        self.check(unsafe { (self.sprite_batch_begin)(batch, info) })
    }

    pub(crate) fn submit_sprite(
        &self,
        batch: sys::CNA_Handle,
        command: &sys::CNA_SpriteCommand,
    ) -> Result<()> {
        // SAFETY: both the owned handle and POD command are live; count is one.
        self.check(unsafe { (self.sprite_batch_submit_many)(batch, command, 1) })
    }

    pub(crate) fn end_sprite_batch(&self, batch: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the wrapper enforces an active begin/end interval.
        self.check(unsafe { (self.sprite_batch_end)(batch) })
    }

    pub(crate) fn destroy_sprite_batch(&self, batch: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the caller transfers exactly-once ownership of a live handle.
        self.check(unsafe { (self.sprite_batch_destroy)(batch) })
    }

    pub(crate) fn keyboard_state(
        &self,
        game: sys::CNA_Handle,
        state: &mut sys::CNA_KeyboardState,
    ) -> Result<()> {
        // SAFETY: the callback-scoped game and output reference are live.
        self.check(unsafe { (self.keyboard_get_state)(game, state) })
    }

    fn last_error_message(&self) -> String {
        let mut required = 0_u64;
        // SAFETY: `required` is a valid output pointer for the duration of the call.
        if unsafe { (self.error_get_last_message_size)(&mut required) } != sys::CNA_RESULT_SUCCESS {
            return "native error details unavailable".to_owned();
        }
        let Ok(capacity) = usize::try_from(required) else {
            return "native error message is too large".to_owned();
        };
        let mut bytes = vec![0_u8; capacity];
        let mut copied = 0_u64;
        // SAFETY: the buffer has `required` writable bytes and remains live for the call.
        let result = unsafe {
            (self.error_copy_last_message)(bytes.as_mut_ptr().cast(), required, &mut copied)
        };
        if result != sys::CNA_RESULT_SUCCESS {
            return "native error details unavailable".to_owned();
        }
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

#[cfg(unix)]
impl Library {
    fn open(path: &Path) -> core::result::Result<Self, String> {
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

    unsafe fn symbol<T: Copy>(&self, name: &'static str) -> Result<T> {
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

fn library_candidates() -> Vec<PathBuf> {
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

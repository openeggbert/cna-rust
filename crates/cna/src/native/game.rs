//! Native game lifecycle calls.

use core::ffi::c_void;

use cna_sys as sys;

use crate::error::{CnaError, ErrorCategory, Result};

use super::Native;

impl Native {
    pub(crate) fn create_game(
        &self,
        info: &sys::CNA_GameCreateInfo,
        handle: &mut sys::CNA_Handle,
    ) -> Result<()> {
        #[cfg(feature = "native-fault-injection")]
        super::fault::check("game-create")?;
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

    pub(crate) fn run_game_one_frame(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the host owns the live handle and invokes the frame on its
        // creation thread, outside a lifecycle callback.
        self.check(unsafe { (self.game_run_one_frame)(game) })
    }

    pub(crate) fn request_game_exit(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: GameContext guarantees a live callback-scoped game handle.
        self.check(unsafe { (self.game_request_exit)(game) })
    }

    pub(crate) fn game_is_active(&self, game: sys::CNA_Handle) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the output is initialized writable storage and the state
        // retains a live parent handle while this synchronous call runs.
        self.check(unsafe { (self.game_get_is_active)(game, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn game_is_mouse_visible(&self, game: sys::CNA_Handle) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: see `game_is_active`.
        self.check(unsafe { (self.game_get_is_mouse_visible)(game, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn set_game_is_mouse_visible(
        &self,
        game: sys::CNA_Handle,
        value: bool,
    ) -> Result<()> {
        // SAFETY: the handle is retained by the active game state.
        self.check(unsafe {
            (self.game_set_is_mouse_visible)(
                game,
                if value { sys::CNA_TRUE } else { sys::CNA_FALSE },
            )
        })
    }

    pub(crate) fn game_is_fixed_time_step(&self, game: sys::CNA_Handle) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: see `game_is_active`.
        self.check(unsafe { (self.game_get_is_fixed_time_step)(game, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn set_game_is_fixed_time_step(
        &self,
        game: sys::CNA_Handle,
        value: bool,
    ) -> Result<()> {
        // SAFETY: the handle is retained by the active game state.
        self.check(unsafe {
            (self.game_set_is_fixed_time_step)(
                game,
                if value { sys::CNA_TRUE } else { sys::CNA_FALSE },
            )
        })
    }

    pub(crate) fn game_target_elapsed_time_ticks(&self, game: sys::CNA_Handle) -> Result<i64> {
        let mut value = 0;
        // SAFETY: see `game_is_active`.
        self.check(unsafe { (self.game_get_target_elapsed_time_ticks)(game, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_game_target_elapsed_time_ticks(
        &self,
        game: sys::CNA_Handle,
        value: i64,
    ) -> Result<()> {
        // SAFETY: the handle is retained by the active game state.
        self.check(unsafe { (self.game_set_target_elapsed_time_ticks)(game, value) })
    }

    pub(crate) fn game_inactive_sleep_time_ticks(&self, game: sys::CNA_Handle) -> Result<i64> {
        let mut value = 0;
        // SAFETY: see `game_is_active`.
        self.check(unsafe { (self.game_get_inactive_sleep_time_ticks)(game, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn set_game_inactive_sleep_time_ticks(
        &self,
        game: sys::CNA_Handle,
        value: i64,
    ) -> Result<()> {
        // SAFETY: the handle is retained by the active game state.
        self.check(unsafe { (self.game_set_inactive_sleep_time_ticks)(game, value) })
    }

    pub(crate) fn reset_game_elapsed_time(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the handle is retained by the active game state.
        self.check(unsafe { (self.game_reset_elapsed_time)(game) })
    }

    pub(crate) fn suppress_game_draw(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the handle is retained by the active game state.
        self.check(unsafe { (self.game_suppress_draw)(game) })
    }

    pub(crate) fn tick_game(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: the handle is retained by the active game state. CNA rejects
        // callback re-entry, and that error is propagated unchanged.
        self.check(unsafe { (self.game_tick)(game) })
    }

    pub(crate) fn set_game_window_title(&self, game: sys::CNA_Handle, title: &str) -> Result<()> {
        let title = sys::CNA_StringView {
            data: title.as_ptr().cast(),
            byte_length: title.len() as u64,
        };
        // SAFETY: the string view borrows UTF-8 bytes for this synchronous call.
        self.check(unsafe { (self.game_set_window_title)(game, title) })
    }

    pub(crate) fn subscribe_game_event(
        &self,
        game: sys::CNA_Handle,
        event: sys::CNA_GameEvent,
        callback: unsafe extern "C" fn(*mut c_void),
        context: *mut c_void,
    ) -> Result<sys::CNA_GameEventRegistrationHandle> {
        let mut registration = sys::CNA_INVALID_HANDLE;
        // SAFETY: the host keeps `context` live until it synchronously
        // unsubscribes the returned registration before destroying the game.
        self.check(unsafe {
            (self.game_subscribe)(game, event, Some(callback), context, &mut registration)
        })?;
        Ok(registration)
    }

    pub(crate) fn subscribe_game_window_event(
        &self,
        game: sys::CNA_Handle,
        event: sys::CNA_GameWindowEvent,
        callback: unsafe extern "C" fn(*mut c_void),
        context: *mut c_void,
    ) -> Result<sys::CNA_GameEventRegistrationHandle> {
        let mut registration = sys::CNA_INVALID_HANDLE;
        // SAFETY: identical registration lifetime to `subscribe_game_event`.
        self.check(unsafe {
            (self.game_window_subscribe)(game, event, Some(callback), context, &mut registration)
        })?;
        Ok(registration)
    }

    pub(crate) fn unsubscribe_game_event(
        &self,
        registration: sys::CNA_GameEventRegistrationHandle,
    ) -> Result<()> {
        // SAFETY: the host releases each owned registration exactly once.
        self.check(unsafe { (self.game_unsubscribe)(registration) })
    }

    pub(crate) fn destroy_game(&self, game: sys::CNA_Handle) -> Result<()> {
        // SAFETY: only the runner calls this for its exactly-once owned handle.
        self.check(unsafe { (self.game_destroy)(game) })?;
        #[cfg(feature = "native-fault-injection")]
        super::fault::check("game-destroy")?;
        Ok(())
    }
}

/// `runtime.h`'s frame-rate arithmetic, launch parameters and title container.
///
/// The launch parameters are the part worth reading first. Before this slice
/// `crates/cna/src/game/services.rs` kept them in a Rust `HashMap` and CNA
/// never heard about them -- the same shape of divergence
/// `graphics_resource.h` had with `Name`. CNA has its own per-game dictionary,
/// populated from the process's arguments by
/// `cna_game_launch_parameters_parse_ext`, and it is the one a real XNA game's
/// command line would land in.
impl Native {
    pub(crate) fn game_clear_to_color(
        &self,
        game: sys::CNA_Handle,
        color: sys::CNA_Color,
    ) -> Result<()> {
        // SAFETY: the game handle is live and the colour is by value.
        self.check(unsafe { (self.game_clear)(game, color) })
    }

    pub(crate) fn game_target_fps(&self, game: sys::CNA_Handle) -> Result<f64> {
        let mut value = 0.0;
        // SAFETY: the game handle is live; the output is a local.
        self.check(unsafe { (self.game_get_target_fps_ext)(game, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn game_target_frame_milliseconds(&self, game: sys::CNA_Handle) -> Result<f64> {
        let mut value = 0.0;
        // SAFETY: the game handle is live; the output is a local.
        self.check(unsafe { (self.game_get_target_ms_frame_time_ext)(game, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn fps_to_frame_milliseconds(&self, fps: i32) -> Result<f64> {
        let mut value = 0.0;
        // SAFETY: the output is a live local; the input is a scalar.
        self.check(unsafe { (self.game_fps_to_milliseconds_per_frame_ext)(fps, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn game_run_application(&self, game: sys::CNA_Handle) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the game handle is live; the output is a local.
        self.check(unsafe { (self.game_get_run_application_ext)(game, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn set_game_run_application(
        &self,
        game: sys::CNA_Handle,
        running: bool,
    ) -> Result<()> {
        let value = if running { sys::CNA_TRUE } else { sys::CNA_FALSE };
        // SAFETY: the game handle is live.
        self.check(unsafe { (self.game_set_run_application_ext)(game, value) })
    }

    pub(crate) fn launch_parameter_count(&self, game: sys::CNA_Handle) -> Result<u64> {
        let mut value = 0;
        // SAFETY: the game handle is live; the output is a local.
        self.check(unsafe { (self.game_launch_parameters_get_count)(game, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn launch_parameters_contain(
        &self,
        game: sys::CNA_Handle,
        key: &str,
    ) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the key outlives the call, which is all the view borrows.
        self.check(unsafe {
            (self.game_launch_parameters_contains_key)(game, string_view(key), &mut value)
        })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn launch_parameter_value(
        &self,
        game: sys::CNA_Handle,
        key: &str,
    ) -> Result<String> {
        let mut count = 0;
        // SAFETY: the key outlives the call; the output is a local.
        self.check(unsafe {
            (self.game_launch_parameters_get_value_size)(game, string_view(key), &mut count)
        })?;
        let mut bytes = vec![0_u8; usize::try_from(count).unwrap_or(0)];
        let mut copied = count;
        // SAFETY: the destination has exactly `count` writable bytes.
        self.check(unsafe {
            (self.game_launch_parameters_copy_value)(
                game,
                string_view(key),
                bytes.as_mut_ptr().cast(),
                count,
                &mut copied,
            )
        })?;
        bytes.truncate(usize::try_from(copied).unwrap_or(0));
        decode_utf8(bytes, "launch-parameter value")
    }

    pub(crate) fn launch_parameter_key(
        &self,
        game: sys::CNA_Handle,
        index: u64,
    ) -> Result<String> {
        let mut count = 0;
        // SAFETY: the game handle is live; the output is a local.
        self.check(unsafe {
            (self.game_launch_parameters_get_key_size)(game, index, &mut count)
        })?;
        let mut bytes = vec![0_u8; usize::try_from(count).unwrap_or(0)];
        let mut copied = count;
        // SAFETY: the destination has exactly `count` writable bytes.
        self.check(unsafe {
            (self.game_launch_parameters_copy_key)(
                game,
                index,
                bytes.as_mut_ptr().cast(),
                count,
                &mut copied,
            )
        })?;
        bytes.truncate(usize::try_from(copied).unwrap_or(0));
        decode_utf8(bytes, "launch-parameter key")
    }

    pub(crate) fn add_launch_parameter(
        &self,
        game: sys::CNA_Handle,
        key: &str,
        value: &str,
    ) -> Result<()> {
        // SAFETY: both strings outlive the call, which is all the views borrow.
        self.check(unsafe {
            (self.game_launch_parameters_add)(game, string_view(key), string_view(value))
        })
    }

    pub(crate) fn parse_launch_parameters(
        &self,
        game: sys::CNA_Handle,
        arguments: &[&str],
    ) -> Result<()> {
        let views: Vec<sys::CNA_StringView> =
            arguments.iter().map(|value| string_view(value)).collect();
        // SAFETY: `views` borrows `arguments`, both outlive the call, and the
        // count is the vector's own length.
        self.check(unsafe {
            (self.game_launch_parameters_parse_ext)(
                game,
                if views.is_empty() {
                    core::ptr::null()
                } else {
                    views.as_ptr()
                },
                views.len() as u64,
            )
        })
    }

    pub(crate) fn title_path(&self, game: sys::CNA_Handle) -> Result<String> {
        let mut count = 0;
        // SAFETY: the game handle is live; the output is a local.
        self.check(unsafe { (self.title_location_get_path_size)(game, &mut count) })?;
        let mut bytes = vec![0_u8; usize::try_from(count).unwrap_or(0)];
        let mut copied = count;
        // SAFETY: the destination has exactly `count` writable bytes.
        self.check(unsafe {
            (self.title_location_copy_path)(
                game,
                bytes.as_mut_ptr().cast(),
                count,
                &mut copied,
            )
        })?;
        bytes.truncate(usize::try_from(copied).unwrap_or(0));
        decode_utf8(bytes, "title path")
    }

    pub(crate) fn set_title_path(&self, game: sys::CNA_Handle, path: &str) -> Result<()> {
        // SAFETY: the path outlives the call, which is all the view borrows.
        self.check(unsafe { (self.title_location_set_path_ext)(game, string_view(path)) })
    }

    /// Reads a file relative to the title location.
    ///
    /// Two calls: the first learns the size with a zero capacity, the second
    /// reads. The header promises `out_bytes` is *always* the file's byte
    /// count, so a too-small buffer is a sizing step rather than a failure --
    /// and it promises no partial write, so the first call cannot corrupt a
    /// destination it was not given.
    pub(crate) fn read_title_file(
        &self,
        game: sys::CNA_Handle,
        path: &str,
    ) -> Result<Vec<u8>> {
        let mut count = 0;
        let sized = unsafe {
            // SAFETY: a null destination is explicitly allowed with a zero
            // capacity, and the output is a live local.
            (self.title_container_read_ext)(
                game,
                string_view(path),
                core::ptr::null_mut(),
                0,
                &mut count,
            )
        };
        if sized != sys::CNA_RESULT_SUCCESS && sized != sys::CNA_RESULT_BUFFER_TOO_SMALL {
            self.check(sized)?;
        }
        let mut bytes = vec![0_u8; usize::try_from(count).unwrap_or(0)];
        let mut read = count;
        // SAFETY: the destination has exactly `count` writable bytes.
        self.check(unsafe {
            (self.title_container_read_ext)(
                game,
                string_view(path),
                bytes.as_mut_ptr(),
                count,
                &mut read,
            )
        })?;
        bytes.truncate(usize::try_from(read).unwrap_or(0));
        Ok(bytes)
    }
}

fn string_view(value: &str) -> sys::CNA_StringView {
    sys::CNA_StringView {
        data: value.as_ptr().cast(),
        byte_length: value.len() as u64,
    }
}

fn decode_utf8(bytes: Vec<u8>, what: &str) -> Result<String> {
    String::from_utf8(bytes).map_err(|_| CnaError::Native {
        code: sys::CNA_RESULT_ENCODING,
        category: ErrorCategory::None,
        message: format!("CNA returned an invalid UTF-8 {what}"),
    })
}

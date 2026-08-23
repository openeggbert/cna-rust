//! Native game-window calls.

use cna_sys as sys;

use crate::error::{CnaError, Result};

use super::Native;

impl Native {
    pub(crate) fn game_window_allow_user_resizing(&self, game: sys::CNA_Handle) -> Result<bool> {
        let mut value = sys::CNA_FALSE;
        // SAFETY: the caller retains the live game and supplies writable output.
        self.check(unsafe { (self.game_window_get_allow_user_resizing)(game, &mut value) })?;
        Ok(value != sys::CNA_FALSE)
    }

    pub(crate) fn set_game_window_allow_user_resizing(
        &self,
        game: sys::CNA_Handle,
        value: bool,
    ) -> Result<()> {
        // SAFETY: the caller retains the live game for this synchronous call.
        self.check(unsafe {
            (self.game_window_set_allow_user_resizing)(
                game,
                if value { sys::CNA_TRUE } else { sys::CNA_FALSE },
            )
        })
    }

    pub(crate) fn game_window_client_bounds(
        &self,
        game: sys::CNA_Handle,
    ) -> Result<sys::CNA_Rectangle> {
        let mut value = sys::CNA_Rectangle::default();
        // SAFETY: the caller retains the live game and supplies writable output.
        self.check(unsafe { (self.game_window_get_client_bounds)(game, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn game_window_current_orientation(
        &self,
        game: sys::CNA_Handle,
    ) -> Result<sys::CNA_DisplayOrientation> {
        let mut value = sys::CNA_DISPLAY_ORIENTATION_DEFAULT;
        // SAFETY: the caller retains the live game and supplies writable output.
        self.check(unsafe { (self.game_window_get_current_orientation)(game, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn game_window_native_handle(&self, game: sys::CNA_Handle) -> Result<u64> {
        let mut value = 0;
        // SAFETY: the caller retains the live game and supplies writable output.
        self.check(unsafe { (self.game_window_get_native_handle)(game, &mut value) })?;
        Ok(value)
    }

    pub(crate) fn game_window_screen_device_name(&self, game: sys::CNA_Handle) -> Result<String> {
        self.game_window_string(
            game,
            self.game_window_get_screen_device_name_size,
            self.game_window_copy_screen_device_name,
        )
    }

    pub(crate) fn game_window_title(&self, game: sys::CNA_Handle) -> Result<String> {
        self.game_window_string(
            game,
            self.game_window_get_title_size,
            self.game_window_copy_title,
        )
    }

    fn game_window_string(
        &self,
        game: sys::CNA_Handle,
        size: sys::cna_game_window_get_title_size_fn,
        copy: sys::cna_game_window_copy_title_fn,
    ) -> Result<String> {
        let mut byte_count = 0;
        // SAFETY: the caller retains the live game and supplies writable output.
        self.check(unsafe { size(game, &mut byte_count) })?;
        let capacity = usize::try_from(byte_count)
            .map_err(|_| CnaError::InvalidInput("window string is too large"))?;
        let mut bytes = vec![0_u8; capacity];
        let mut copied = 0;
        // SAFETY: `bytes` supplies exactly its reported capacity for the
        // synchronous copy and `copied` is writable output.
        self.check(unsafe { copy(game, bytes.as_mut_ptr().cast(), byte_count, &mut copied) })?;
        debug_assert_eq!(copied, byte_count);
        String::from_utf8(bytes).map_err(|_| CnaError::Native {
            code: sys::CNA_RESULT_ENCODING,
            message: "CNA returned a non-UTF-8 window string".to_owned(),
        })
    }

    pub(crate) fn begin_game_screen_device_change(
        &self,
        game: sys::CNA_Handle,
        fullscreen: bool,
    ) -> Result<()> {
        // SAFETY: the caller retains the live game for this synchronous call.
        self.check(unsafe {
            (self.game_window_begin_screen_device_change)(
                game,
                if fullscreen {
                    sys::CNA_TRUE
                } else {
                    sys::CNA_FALSE
                },
            )
        })
    }

    pub(crate) fn end_game_screen_device_change(
        &self,
        game: sys::CNA_Handle,
        screen: &str,
        width: i32,
        height: i32,
    ) -> Result<()> {
        let screen = sys::CNA_StringView {
            data: screen.as_ptr().cast(),
            byte_length: screen.len() as u64,
        };
        // SAFETY: the string view borrows UTF-8 bytes for this synchronous call.
        self.check(unsafe {
            (self.game_window_end_screen_device_change)(game, screen, width, height)
        })
    }
}

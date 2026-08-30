//! CNA result-code and native diagnostic translation.

use cna_sys as sys;

use crate::error::{CnaError, ErrorCategory, Result};

use super::Native;

impl Native {
    pub(crate) fn check(&self, result: sys::CNA_Result) -> Result<()> {
        if result == sys::CNA_RESULT_SUCCESS {
            return Ok(());
        }
        Err(CnaError::Native {
            code: result,
            category: self.last_error_category(),
            message: self.last_error_message(),
        })
    }

    /// CNA's own classification of the last failure on this thread.
    ///
    /// Reading it must not disturb the message: the canonical route refuses an
    /// invalid descriptor without overwriting the thread-local diagnostic, so
    /// an unreadable category degrades to `None` rather than costing the text.
    fn last_error_category(&self) -> ErrorCategory {
        let mut info = sys::CNA_ErrorInfo {
            struct_size: core::mem::size_of::<sys::CNA_ErrorInfo>() as u32,
            struct_version: sys::CNA_ERROR_INFO_STRUCT_VERSION,
            ..sys::CNA_ErrorInfo::default()
        };
        // SAFETY: the descriptor is a caller-owned versioned output whose
        // prefix this build declares exactly.
        if unsafe { (self.error_get_last_info)(&mut info) } != sys::CNA_RESULT_SUCCESS {
            return ErrorCategory::None;
        }
        ErrorCategory::from_native(info.category)
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

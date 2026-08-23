//! CNA result-code and native diagnostic translation.

use cna_sys as sys;

use crate::error::{CnaError, Result};

use super::Native;

impl Native {
    pub(crate) fn check(&self, result: sys::CNA_Result) -> Result<()> {
        if result == sys::CNA_RESULT_SUCCESS {
            return Ok(());
        }
        Err(CnaError::Native {
            code: result,
            message: self.last_error_message(),
        })
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

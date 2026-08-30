//! CNA's process-wide log, its level filter, and a Rust sink.
//!
//! XNA has no logging surface at all, so the whole family is a CNA concept.
//!
//! The destination is a correctness matter rather than a preference: CNA's
//! default sink writes to **stderr**, never stdout, because a terminal-hosted
//! game draws its frame on stdout and a log line there would corrupt it. That
//! is the reason a replaceable sink exists.

#![allow(clippy::missing_errors_doc)]

use core::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

use cna_sys as sys;

use crate::error::Result;
use crate::native::Native;

/// Severity of one log line.
///
/// `Experiment` is deliberately 100 rather than 6: it sits below every
/// ordinary level so enabling it turns on everything.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LogLevel {
    Fatal,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    Experiment,
    /// A level a newer CNA introduced.
    Unrecognized(u32),
}

/// Which part of CNA a log line came from.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum LogCategory {
    Application,
    Error,
    System,
    Audio,
    Video,
    Render,
    Input,
    Test,
    Gpu,
    /// A category a newer CNA introduced.
    Unrecognized(u32),
}

impl LogLevel {
    const fn from_native(value: sys::CNA_LogLevel) -> Self {
        match value {
            sys::CNA_LOG_LEVEL_FATAL => Self::Fatal,
            sys::CNA_LOG_LEVEL_ERROR => Self::Error,
            sys::CNA_LOG_LEVEL_WARN => Self::Warn,
            sys::CNA_LOG_LEVEL_INFO => Self::Info,
            sys::CNA_LOG_LEVEL_DEBUG => Self::Debug,
            sys::CNA_LOG_LEVEL_TRACE => Self::Trace,
            sys::CNA_LOG_LEVEL_EXPERIMENT => Self::Experiment,
            other => Self::Unrecognized(other),
        }
    }

    const fn as_native(self) -> sys::CNA_LogLevel {
        match self {
            Self::Fatal => sys::CNA_LOG_LEVEL_FATAL,
            Self::Error => sys::CNA_LOG_LEVEL_ERROR,
            Self::Warn => sys::CNA_LOG_LEVEL_WARN,
            Self::Info => sys::CNA_LOG_LEVEL_INFO,
            Self::Debug => sys::CNA_LOG_LEVEL_DEBUG,
            Self::Trace => sys::CNA_LOG_LEVEL_TRACE,
            Self::Experiment => sys::CNA_LOG_LEVEL_EXPERIMENT,
            Self::Unrecognized(value) => value,
        }
    }
}

impl LogCategory {
    const fn from_native(value: sys::CNA_LogCategory) -> Self {
        match value {
            sys::CNA_LOG_CATEGORY_APPLICATION => Self::Application,
            sys::CNA_LOG_CATEGORY_ERROR => Self::Error,
            sys::CNA_LOG_CATEGORY_SYSTEM => Self::System,
            sys::CNA_LOG_CATEGORY_AUDIO => Self::Audio,
            sys::CNA_LOG_CATEGORY_VIDEO => Self::Video,
            sys::CNA_LOG_CATEGORY_RENDER => Self::Render,
            sys::CNA_LOG_CATEGORY_INPUT => Self::Input,
            sys::CNA_LOG_CATEGORY_TEST => Self::Test,
            sys::CNA_LOG_CATEGORY_GPU => Self::Gpu,
            other => Self::Unrecognized(other),
        }
    }

    const fn as_native(self) -> sys::CNA_LogCategory {
        match self {
            Self::Application => sys::CNA_LOG_CATEGORY_APPLICATION,
            Self::Error => sys::CNA_LOG_CATEGORY_ERROR,
            Self::System => sys::CNA_LOG_CATEGORY_SYSTEM,
            Self::Audio => sys::CNA_LOG_CATEGORY_AUDIO,
            Self::Video => sys::CNA_LOG_CATEGORY_VIDEO,
            Self::Render => sys::CNA_LOG_CATEGORY_RENDER,
            Self::Input => sys::CNA_LOG_CATEGORY_INPUT,
            Self::Test => sys::CNA_LOG_CATEGORY_TEST,
            Self::Gpu => sys::CNA_LOG_CATEGORY_GPU,
            Self::Unrecognized(value) => value,
        }
    }
}

/// A destination for CNA's log lines.
///
/// A sink must return normally and must not call back into CNA. A panic is
/// contained at the FFI boundary rather than unwinding through C, and the sink
/// that panicked is uninstalled so one bad line does not repeat for the life
/// of the process; [`sink_panicked`] reports that it happened.
pub trait LogSink: Send + 'static {
    /// Receives one formatted line, without a trailing newline.
    fn write(&mut self, level: LogLevel, category: LogCategory, message: &str);
}

impl<F> LogSink for F
where
    F: FnMut(LogLevel, LogCategory, &str) + Send + 'static,
{
    fn write(&mut self, level: LogLevel, category: LogCategory, message: &str) {
        self(level, category, message);
    }
}

/// The installed sink.
///
/// CNA takes a caller-owned context pointer, and this deliberately passes null
/// instead: the sink lives here, so there is no pointer that can dangle across
/// a replacement, and the trampoline has nothing to validate.
static SINK: Mutex<Option<Box<dyn LogSink>>> = Mutex::new(None);
static PANICKED: Mutex<Option<String>> = Mutex::new(None);

unsafe extern "C" fn sink_trampoline(
    level: sys::CNA_LogLevel,
    category: sys::CNA_LogCategory,
    message: sys::CNA_StringView,
    context: *mut c_void,
) {
    let _ = context;
    // A sink that logs would re-enter this lock. CNA forbids calling back into
    // it, so rather than deadlock on a contract violation the line is dropped.
    let Ok(mut installed) = SINK.try_lock() else {
        return;
    };
    let Some(sink) = installed.as_mut() else {
        return;
    };
    let length = usize::try_from(message.byte_length).unwrap_or(0);
    let text = if message.data.is_null() || length == 0 {
        String::new()
    } else {
        // SAFETY: CNA documents the bytes as counted UTF-8 borrowed for the
        // duration of this call; they are copied before it returns.
        let bytes = unsafe { core::slice::from_raw_parts(message.data.cast::<u8>(), length) };
        String::from_utf8_lossy(bytes).into_owned()
    };
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        sink.write(
            LogLevel::from_native(level),
            LogCategory::from_native(category),
            &text,
        );
    }));
    if outcome.is_err() {
        *installed = None;
        *PANICKED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Some("a Rust log sink panicked and was uninstalled".to_owned());
    }
}

fn string_view(value: &str) -> sys::CNA_StringView {
    sys::CNA_StringView {
        data: value.as_ptr().cast::<core::ffi::c_char>(),
        byte_length: value.len() as u64,
    }
}

/// Installs a Rust sink in place of CNA's default stderr destination.
pub fn set_sink(sink: Box<dyn LogSink>) -> Result<()> {
    let native = Native::process()?;
    *SINK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(sink);
    // SAFETY: the trampoline has the audited signature and ignores the context
    // pointer, so no Rust address is handed to CNA.
    let result = unsafe {
        (native.runtime.logger_set_sink)(Some(sink_trampoline), core::ptr::null_mut())
    };
    if let Err(error) = native.check(result) {
        *SINK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        return Err(error);
    }
    Ok(())
}

/// Restores CNA's default stderr sink and drops the Rust one.
///
/// The native reset happens first, so there is no window in which CNA could
/// call a trampoline whose sink has already been dropped.
pub fn reset_sink() -> Result<()> {
    let native = Native::process()?;
    // SAFETY: the route takes no arguments and restores CNA's own sink.
    native.check(unsafe { (native.runtime.logger_reset_sink)() })?;
    *SINK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    Ok(())
}

/// Reports and clears a contained sink panic, if one occurred.
pub fn sink_panicked() -> Option<String> {
    PANICKED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}

/// The lowest severity CNA currently emits.
pub fn minimum_level() -> Result<LogLevel> {
    let native = Native::process()?;
    let mut value = 0;
    // SAFETY: the output is a live local of the declared type.
    native.check(unsafe { (native.runtime.logger_get_minimum_level)(&mut value) })?;
    Ok(LogLevel::from_native(value))
}

/// Sets the lowest severity CNA emits.
pub fn set_minimum_level(level: LogLevel) -> Result<()> {
    let native = Native::process()?;
    // SAFETY: the level is a plain fixed-width value.
    native.check(unsafe { (native.runtime.logger_set_minimum_level)(level.as_native()) })
}

/// Writes one line at the given level and category.
pub fn log(level: LogLevel, category: LogCategory, message: &str) -> Result<()> {
    let native = Native::process()?;
    let view = string_view(message);
    // SAFETY: `view` borrows `message` for the duration of the call.
    native.check(unsafe {
        (native.runtime.logger_log)(level.as_native(), view, category.as_native(), sys::CNA_TRUE)
    })
}

/// Writes one line only when `condition` holds.
pub fn log_if(level: LogLevel, category: LogCategory, message: &str, condition: bool) -> Result<()> {
    let native = Native::process()?;
    let view = string_view(message);
    let condition = if condition {
        sys::CNA_TRUE
    } else {
        sys::CNA_FALSE
    };
    // SAFETY: `view` borrows `message` for the duration of the call.
    native.check(unsafe {
        (native.runtime.logger_log)(level.as_native(), view, category.as_native(), condition)
    })
}

macro_rules! level_route {
    ($name:ident, $slot:ident, $conditional:ident, $conditional_slot:ident, $level:literal) => {
        #[doc = concat!("Writes one ", $level, " line in the given category.")]
        pub fn $name(message: &str, category: LogCategory) -> Result<()> {
            let native = Native::process()?;
            let view = string_view(message);
            // SAFETY: `view` borrows `message` for the duration of the call.
            native.check(unsafe { (native.runtime.$slot)(view, category.as_native()) })
        }

        #[doc = concat!("Writes one ", $level, " line only when `condition` holds.")]
        pub fn $conditional(message: &str, condition: bool) -> Result<()> {
            let native = Native::process()?;
            let view = string_view(message);
            let condition = if condition {
                sys::CNA_TRUE
            } else {
                sys::CNA_FALSE
            };
            // SAFETY: `view` borrows `message` for the duration of the call.
            native.check(unsafe { (native.runtime.$conditional_slot)(view, condition) })
        }
    };
}

level_route!(fatal, logger_fatal, fatal_if, logger_fatal_if, "fatal");
level_route!(error, logger_error, error_if, logger_error_if, "error");
level_route!(warn, logger_warn, warn_if, logger_warn_if, "warning");
level_route!(info, logger_info, info_if, logger_info_if, "informational");
level_route!(debug, logger_debug, debug_if, logger_debug_if, "debug");
level_route!(trace, logger_trace, trace_if, logger_trace_if, "trace");
level_route!(
    experiment,
    logger_experiment,
    experiment_if,
    logger_experiment_if,
    "experiment"
);

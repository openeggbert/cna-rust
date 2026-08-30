//! CNA's process-wide log, its level filter, and a Rust sink.
//!
//! The sink is process-global upstream, so everything runs in one test.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use cna::extensions::logging::{
    error, info, info_if, log, minimum_level, reset_sink, set_minimum_level, set_sink,
    sink_panicked, trace, LogCategory, LogLevel,
};

#[derive(Default)]
struct Recorder {
    lines: Arc<Mutex<Vec<(LogLevel, LogCategory, String)>>>,
}

impl cna::extensions::logging::LogSink for Recorder {
    fn write(&mut self, level: LogLevel, category: LogCategory, message: &str) {
        self.lines
            .lock()
            .unwrap()
            .push((level, category, message.to_owned()));
    }
}

#[test]
fn logging_level_filter_and_rust_sink() {
    let original = minimum_level().expect("CNA reports its minimum level");

    let lines = Arc::new(Mutex::new(Vec::new()));
    set_sink(Box::new(Recorder {
        lines: Arc::clone(&lines),
    }))
    .expect("a Rust sink replaces CNA's stderr default");

    set_minimum_level(LogLevel::Trace).expect("every level is emitted");
    assert_eq!(minimum_level(), Ok(LogLevel::Trace));

    info("first line", LogCategory::Test).expect("info");
    error("second line", LogCategory::Application).expect("error");
    log(LogLevel::Debug, LogCategory::Render, "third line").expect("log");
    // A conditional route writes nothing when the condition is false.
    info_if("not written", false).expect("info_if false");
    info_if("fourth line", true).expect("info_if true");

    {
        let recorded = lines.lock().unwrap();
        let messages: Vec<&str> = recorded.iter().map(|(_, _, text)| text.as_str()).collect();
        assert!(messages.iter().any(|text| text.contains("first line")));
        assert!(messages.iter().any(|text| text.contains("second line")));
        assert!(messages.iter().any(|text| text.contains("third line")));
        assert!(messages.iter().any(|text| text.contains("fourth line")));
        assert!(
            !messages.iter().any(|text| text.contains("not written")),
            "a false condition must not reach the sink",
        );
        assert!(recorded
            .iter()
            .any(|(level, category, _)| *level == LogLevel::Info
                && *category == LogCategory::Test));
    }

    // The level filter is CNA's, not the sink's: raising it stops delivery.
    let before = lines.lock().unwrap().len();
    set_minimum_level(LogLevel::Error).expect("raise the minimum level");
    trace("filtered away", LogCategory::Test).expect("trace");
    assert_eq!(lines.lock().unwrap().len(), before);
    error("still delivered", LogCategory::Test).expect("error");
    assert_eq!(lines.lock().unwrap().len(), before + 1);

    // A panicking sink is contained at the FFI boundary and uninstalled, so one
    // bad line does not repeat for the life of the process.
    set_minimum_level(LogLevel::Trace).expect("lower the minimum level");
    let calls = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&calls);
    set_sink(Box::new(move |_: LogLevel, _: LogCategory, _: &str| {
        counted.fetch_add(1, Ordering::SeqCst);
        panic!("sink under test");
    }))
    .expect("install the panicking sink");
    assert_eq!(sink_panicked(), None);
    info("panics once", LogCategory::Test).expect("the panic never reaches this caller");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(sink_panicked().is_some());
    info("no second call", LogCategory::Test).expect("the sink is gone");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(sink_panicked(), None);

    reset_sink().expect("CNA's default sink is restored");
    set_minimum_level(original).expect("restore the original level");
}

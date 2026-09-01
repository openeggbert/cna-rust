//! The desktop devices, driven through their substitute backends.
//!
//! No machine this runs on has a system tray, a message box a user can answer,
//! a file picker or a vibration motor. Every one of these families ships a
//! substitute backend and a test log for exactly that reason, and binding them
//! is what turns "the route exists" into "the game asked for what it meant to".
//!
//! What each test asserts is the *game's own request*, read back out of the
//! log: the severity a message box carried, the motor values a rumble asked
//! for, the paths a dialog answered with. None of it needs a desktop session.

use std::sync::{Arc, Mutex};

use cna::extensions::devices::{
    device_type, file_dialog, message_box, try_set_clipboard_text, vibrate_controller, DeviceType,
    FileDialogFilter, MessageBoxType, SystemTray,
};
use cna::Microsoft::Xna::Framework::{Game, GameContext};
use cna::{run_for_frames, GameState, GameStateAccess, Result};

#[derive(Debug, Default)]
struct Observed {
    device_type: Option<DeviceType>,
    clipboard_taken: Option<bool>,
    message_box_supported: bool,
    message_box_choice: Option<Option<usize>>,
    message_box_log: Option<cna::extensions::devices::MessageBoxTestLog>,
    dialog_supported: bool,
    dialog_paths: Vec<String>,
    vibration_supported_before: bool,
    vibration_supported_after: bool,
    vibration_name: Option<String>,
    vibration_log: Option<cna::extensions::devices::VibrationTestLog>,
    tray_supported: bool,
    tray_entry_label_readback: Option<(bool, bool)>,
    tray_clicks: u32,
    notes: Vec<String>,
}

#[derive(Default)]
struct DeviceGame {
    state: Arc<GameState>,
    observed: Arc<Mutex<Observed>>,
}

impl GameStateAccess for DeviceGame {
    fn game_state(&self) -> &Arc<GameState> {
        &self.state
    }
}

impl Game for DeviceGame {
    fn LoadContent(&mut self, game: &mut GameContext<'_>) -> Result<()> {
        let mut observed = Observed::default();

        observed.device_type = device_type().ok();
        observed.clipboard_taken = try_set_clipboard_text(game, "cna-rust").ok();

        // --- message boxes -------------------------------------------------
        observed.message_box_supported = message_box::is_supported(game).unwrap_or(false);
        if message_box::set_test_backend(game, true, 1).is_ok() {
            message_box::show(game, MessageBoxType::Warning, "title", "body")
                .expect("a simple box behind the substitute backend");
            observed.message_box_choice = message_box::show_choice(
                game,
                MessageBoxType::Information,
                "pick",
                "one",
                &["yes", "no", "maybe"],
            )
            .ok();
            observed.message_box_log = message_box::test_log(game).ok();
            let _ = message_box::set_test_backend(game, false, 0);
        } else {
            observed.notes.push("no message-box backend here".to_owned());
        }

        // --- file dialogs --------------------------------------------------
        observed.dialog_supported = file_dialog::is_supported(game).unwrap_or(false);
        let answered: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        if file_dialog::set_test_backend(game, true, &["/tmp/one.png", "/tmp/two.png"]).is_ok() {
            let sink = Arc::clone(&answered);
            file_dialog::show_open_file(
                game,
                &[FileDialogFilter {
                    name: "PNG image".to_owned(),
                    pattern: "*.png".to_owned(),
                }],
                "/tmp",
                true,
                move |paths| {
                    *sink
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = paths;
                },
            )
            .expect("an open dialog behind the substitute backend");
            observed.dialog_paths = answered
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            let _ = file_dialog::set_test_backend(game, false, &[]);
        } else {
            observed.notes.push("no file-dialog backend here".to_owned());
        }

        // --- the vibration motor -------------------------------------------
        observed.vibration_supported_before =
            vibrate_controller::is_supported(game).unwrap_or(false);
        if vibrate_controller::set_test_backend(game, true, true, "Test Motor").is_ok() {
            observed.vibration_supported_after =
                vibrate_controller::is_supported(game).unwrap_or(false);
            observed.vibration_name = vibrate_controller::device_name(game).ok();
            vibrate_controller::start(game, 5_000_000).expect("a plain rumble");
            vibrate_controller::start_with_intensity(game, 2_500_000, 0.75)
                .expect("an intensity rumble");
            vibrate_controller::start_left_right(game, 0.25, 0.5, 1_000_000)
                .expect("a two-motor rumble");
            vibrate_controller::stop(game).expect("stop");
            observed.vibration_log = vibrate_controller::test_log(game).ok();
            let _ = vibrate_controller::set_test_backend(game, false, false, "");
        } else {
            observed.notes.push("no vibration backend here".to_owned());
        }

        // --- the system tray ------------------------------------------------
        observed.tray_supported = SystemTray::is_supported(game).unwrap_or(false);
        match SystemTray::with_test_backend(game, "cna-rust") {
            Ok(tray) => {
                let clicks = Arc::new(std::sync::atomic::AtomicU32::new(0));
                let counter = Arc::clone(&clicks);
                let entry = tray
                    .add_entry("Quit", true, true, false, move || {
                        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    })
                    .expect("a menu entry");
                tray.set_entry_label(entry, "Exit").expect("a new label");
                tray.set_entry_enabled(entry, false).expect("disable");
                tray.set_entry_checked(entry, true).expect("check");
                observed.tray_entry_label_readback = Some((
                    tray.entry_enabled(entry).expect("enabled"),
                    tray.entry_checked(entry).expect("checked"),
                ));
                tray.click_entry_for_tests(entry).expect("a synthetic click");
                observed.tray_clicks = clicks.load(std::sync::atomic::Ordering::SeqCst);
                tray.set_tooltip("cna-rust, still running")
                    .expect("a new tooltip");
                // Dropping the tray frees the handler *after* removing the
                // icon; a click here would be a use-after-free if the order
                // were wrong.
                drop(tray);
            }
            Err(error) => observed.notes.push(format!("no tray here: {error}")),
        }

        *self
            .observed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = observed;
        Ok(())
    }
}

#[test]
fn the_desktop_devices_answer_through_their_substitute_backends() {
    if std::env::var_os("CNA_NATIVE_LIBRARY").is_none() {
        return;
    }
    let observed = Arc::new(Mutex::new(Observed::default()));
    let game = DeviceGame {
        state: Arc::new(GameState::default()),
        observed: Arc::clone(&observed),
    };
    run_for_frames(game, 1).expect("one frame driving the device layer");
    let observed = observed
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for note in &observed.notes {
        println!("NOTE: {note}");
    }
    println!(
        "NOTE: device_type={:?} clipboard_taken={:?} \
         supported: message_box={} dialog={} tray={}",
        observed.device_type,
        observed.clipboard_taken,
        observed.message_box_supported,
        observed.dialog_supported,
        observed.tray_supported
    );

    // A device kind is always answerable: it is a property of the process.
    assert!(
        observed.device_type.is_some(),
        "the device kind should answer even with no device layer"
    );

    // Each block below runs only when its substitute backend installed. Say so
    // out loud: a run where none of them did would otherwise pass on the one
    // assertion above, and look like it had measured something.
    println!(
        "MEASURED: message_box={} file_dialog={} vibration={} tray={}",
        observed.message_box_log.is_some(),
        !observed.dialog_paths.is_empty(),
        observed.vibration_log.is_some(),
        observed.tray_entry_label_readback.is_some()
    );
    let measured = usize::from(observed.message_box_log.is_some())
        + usize::from(!observed.dialog_paths.is_empty())
        + usize::from(observed.vibration_log.is_some())
        + usize::from(observed.tray_entry_label_readback.is_some());
    assert!(
        measured > 0,
        "no substitute backend installed, so this test measured nothing. Every one \
         of these families ships one, so this is a real failure rather than a skip"
    );

    if let Some(log) = observed.message_box_log {
        // The substitute backend was told to choose button 1, so that is what
        // the game must have been told.
        assert_eq!(
            observed.message_box_choice,
            Some(Some(1)),
            "the chosen button should be the one the backend was set to"
        );
        assert_eq!(log.simple_calls, 1, "one simple box was shown");
        assert_eq!(log.choice_calls, 1, "one choice box was shown");
        assert_eq!(
            log.last_type,
            Some(MessageBoxType::Information),
            "the last box carried the severity the game passed"
        );
        assert_eq!(
            log.last_button_count, 3,
            "and the three buttons the game passed"
        );
    }

    if !observed.dialog_paths.is_empty() {
        assert_eq!(
            observed.dialog_paths,
            vec!["/tmp/one.png".to_owned(), "/tmp/two.png".to_owned()],
            "the handler should receive exactly the paths the backend was set to"
        );
    }

    if let Some(log) = observed.vibration_log {
        assert!(
            !observed.vibration_supported_before,
            "this host is expected to have no real vibration motor"
        );
        assert!(
            observed.vibration_supported_after,
            "the substitute motor should report itself supported"
        );
        assert_eq!(
            observed.vibration_name.as_deref(),
            Some("Test Motor"),
            "and should carry the name it was installed with"
        );
        assert_eq!(log.start_calls, 2, "two single-motor starts");
        assert_eq!(log.left_right_calls, 1, "one two-motor start");
        assert_eq!(log.stop_calls, 1);
        assert_eq!(
            log.last_large_motor, 0.25,
            "the large motor value must survive the crossing"
        );
        assert_eq!(log.last_small_motor, 0.5);
        assert_eq!(log.last_duration_ticks, 1_000_000);
    }

    if let Some((enabled, checked)) = observed.tray_entry_label_readback {
        assert!(!enabled, "the entry was disabled and should read back so");
        assert!(checked, "the entry was checked and should read back so");
        assert_eq!(
            observed.tray_clicks, 1,
            "a synthetic click should reach the entry's handler exactly once"
        );
    }
}

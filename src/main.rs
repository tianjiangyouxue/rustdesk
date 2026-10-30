#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use librustdesk::*;

// Allow isolated parallel instances (e.g. one per RustDesk server) on the same
// machine. The instance name drives config dir, tray mutex, IPC names and log dir.
fn apply_app_name_override() {
    if let Ok(name) = std::env::var("RUSTDESK_APP_NAME") {
        let name = name.trim();
        if !name.is_empty() {
            *hbb_common::config::APP_NAME.write().unwrap() = name.to_owned();
        }
    }
}

#[cfg(any(target_os = "android", target_os = "ios", feature = "flutter"))]
fn main() {
    apply_app_name_override();
    if !common::global_init() {
        eprintln!("Global initialization failed.");
        return;
    }
    common::test_rendezvous_server();
    common::test_nat_type();
    common::global_clean();
}

#[cfg(not(any(target_os = "android", target_os = "ios", feature = "flutter")))]
fn main() -> std::process::ExitCode {
    apply_app_name_override();
    #[cfg(windows)]
    if let Some(exit_code) = platform::handle_service_control() {
        return exit_code;
    }
    #[cfg(all(windows, not(feature = "inline")))]
    unsafe {
        winapi::um::shellscalingapi::SetProcessDpiAwareness(2);
    }
    // Preserve command failures, notably verified macOS offline update failures,
    // as a non-zero process exit status for CLI callers.
    let exit_code = match crate::core_main::core_main() {
        Some(crate::core_main::CoreMainAction::StartUi(mut args)) => {
            ui::start(&mut args);
            std::process::ExitCode::SUCCESS
        }
        Some(crate::core_main::CoreMainAction::ExitFailure(err)) => {
            eprintln!("Command failed: {err}");
            hbb_common::log::error!("Command failed: {err}");
            std::process::ExitCode::FAILURE
        }
        None => std::process::ExitCode::SUCCESS,
    };
    common::global_clean();
    exit_code
}

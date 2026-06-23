//! Indiana menulet — Tauri 2 system tray app.
//! Thin face onto the Indiana daemon; shows, never computes.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::{
    image::Image,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

mod socket;

/// Tracks whether we spawned the daemon, and whether a native dialog is open.
pub(crate) struct DaemonState {
    pub(crate) ours: bool,
    pub(crate) dialog_open: bool,
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(Mutex::new(DaemonState { ours: false, dialog_open: false }))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let icon = Image::from_bytes(include_bytes!("../icons/tray.png"))
                .expect("tray icon not found");

            let window = app.get_webview_window("main").unwrap();

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .icon_as_template(true)
                .on_tray_icon_event(move |tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let w = tray.app_handle().get_webview_window("main").unwrap();
                        if w.is_visible().unwrap_or(false) {
                            let _ = w.hide();
                        } else {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            let wh = window.clone();
            let handle = app.handle().clone();
            window.on_window_event(move |event| {
                if let WindowEvent::Focused(false) = event {
                    if handle
                        .try_state::<Mutex<DaemonState>>()
                        .map(|s| s.lock().unwrap().dialog_open)
                        .unwrap_or(false)
                    {
                        return;
                    }
                    let _ = wh.hide();
                }
            });

            // M12.5.1 — Connect-or-spawn on launch.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                socket::spawn_daemon(&handle);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            socket::commands::status,
            socket::commands::add_folder,
            socket::commands::remove_folder,
            socket::commands::copy_folder,
            socket::commands::shutdown,
            socket::commands::spawn_sidecar,
            socket::commands::read_focus,
            socket::commands::save_focus,
            socket::commands::daemon_is_ours,
            socket::commands::home_dir,
            socket::commands::set_dialog_open,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

//! Indiana menulet — Tauri 2 system tray app.
//! Thin face onto the Indiana daemon; shows, never computes.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::{
    image::Image,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
use tauri_plugin_shell::ShellExt;

mod protocol;
mod socket;

/// Tracks whether we spawned the daemon (so we know whether to offer stop).
pub(crate) struct DaemonState {
    pub(crate) ours: bool,
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(Mutex::new(DaemonState { ours: false }))
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
            window.on_window_event(move |event| {
                if let WindowEvent::Focused(false) = event {
                    let _ = wh.hide();
                }
            });

            // M12.5.1 — Connect-or-spawn on launch.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                // Check if a daemon is already alive.
                if socket::status().is_some() {
                    return;
                }

                // No daemon → try spawning the sidecar.
                let shell = match handle.shell().sidecar("indiana") {
                    Ok(s) => s,
                    Err(_) => return,
                };
                let (_rx, _child) = match shell.args(["serve"]).spawn() {
                    Ok(s) => s,
                    Err(_) => return,
                };

                // Mark daemon as ours (we spawned it).
                if let Some(state) = handle.try_state::<Mutex<DaemonState>>() {
                    state.lock().unwrap().ours = true;
                }

                // Poll for daemon to come up.
                for _ in 0..20 {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    if socket::status().is_some() {
                        break;
                    }
                }
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

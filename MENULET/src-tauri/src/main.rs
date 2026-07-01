//! Indiana menulet — Tauri 2 system tray app.
//! Thin face onto the Indiana daemon; shows, never computes.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// tauri-nspanel re-exports the deprecated `cocoa` crate, and its `panel_delegate!`
// macro expands to `cocoa` types (`id`/`nil`) plus a `cargo-clippy` cfg check.
// Both are upstream and unavoidable until the plugin migrates to objc2-app-kit,
// so silence the resulting `deprecated` / `unexpected_cfgs` noise here.
#![allow(deprecated, unexpected_cfgs)]

use std::sync::Mutex;
use tauri::{
    image::Image,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, PhysicalPosition, Rect, WebviewWindow,
};
use tauri_nspanel::{
    cocoa::appkit::NSWindowCollectionBehavior, panel_delegate, ManagerExt, WebviewWindowExt,
};

mod socket;

/// Tracks whether a native dialog is open (so focus loss won't hide the panel).
/// Daemon stoppability is reported by the daemon itself (StatusResponse), so the
/// menulet keeps no lifecycle/ownership state of its own.
pub(crate) struct DaemonState {
    pub(crate) dialog_open: bool,
}

/// Float above normal windows (NSFloatingWindowLevel region).
#[allow(non_upper_case_globals)]
const PANEL_LEVEL: i32 = 4;
/// NSWindowStyleMaskNonactivatingPanel — panel never activates the app.
#[allow(non_upper_case_globals)]
const NONACTIVATING_PANEL_MASK: i32 = 1 << 7;

/// 32×32 template icon: two colons ("::"). Template (alpha mask):
/// macOS tints black pixels to the menu bar color; transparent stays transparent.
fn build_icon() -> Image<'static> {
    const W: u32 = 32;
    const H: u32 = 32;
    // Four dots — two colons, each colon = two vertically stacked dots.
    // Centers at x ∈ {9, 22}, y ∈ {9, 21}. Radius² = 5 (≈2.24 px radius, ~5 px diameter).
    const CENTERS: [(i32, i32); 4] = [(9, 9), (9, 21), (22, 9), (22, 21)];
    const R2: i32 = 5;

    let mut pixels = Vec::with_capacity((W * H * 4) as usize);
    for y in 0..H {
        for x in 0..W {
            let on = CENTERS
                .iter()
                .any(|(cx, cy)| (x as i32 - cx).pow(2) + (y as i32 - cy).pow(2) <= R2);
            if on {
                pixels.extend_from_slice(&[0, 0, 0, 255]);
            } else {
                pixels.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    Image::new_owned(pixels, W, H)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_nspanel::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(Mutex::new(DaemonState { dialog_open: false }))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let icon = build_icon();

            let window = app.get_webview_window("main").unwrap();

            // Convert the pre-created window to a non-activating NSPanel so it
            // never steals focus (no app-switch flash) and joins whatever Space
            // is active, including over fullscreen apps — no Space-switch jump.
            let panel = window.to_panel().unwrap();
            panel.set_level(PANEL_LEVEL);
            panel.set_style_mask(NONACTIVATING_PANEL_MASK);
            // Join all Spaces and sit over fullscreen apps so the panel appears
            // on whatever desktop is active, with no Space-switch animation.
            panel.set_collection_behaviour(
                NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                    | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary,
            );

            // Hide on blur. A non-activating panel never becomes key the way a
            // normal window does, so WindowEvent::Focused(false) won't fire —
            // use the panel's resign-key delegate instead. Respect the dialog
            // guard so the native folder picker doesn't dismiss the panel.
            let handle = app.handle().clone();
            let delegate = panel_delegate!(IndianaPanelDelegate {
                window_did_resign_key
            });
            delegate.set_listener(Box::new(move |name: String| {
                if name != "window_did_resign_key" {
                    return;
                }
                let dialog_open = handle
                    .try_state::<Mutex<DaemonState>>()
                    .map(|s| s.lock().unwrap().dialog_open)
                    .unwrap_or(false);
                if dialog_open {
                    return;
                }
                if let Ok(panel) = handle.get_webview_panel("main") {
                    panel.order_out(None);
                }
            }));
            panel.set_delegate(delegate);

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .icon_as_template(true)
                .on_tray_icon_event(move |tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        position,
                        rect,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        let Ok(panel) = app.get_webview_panel("main") else {
                            return;
                        };
                        if panel.is_visible() {
                            panel.order_out(None);
                            return;
                        }
                        if let Some(window) = app.get_webview_window("main") {
                            position_panel(&window, &rect, position);
                        }
                        panel.show();
                    }
                })
                .build(app)?;

            // M12.5.1 — Connect-or-spawn on launch.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                if let Err(e) = socket::spawn_daemon(&handle) {
                    eprintln!("indiana: auto-spawn failed: {e}");
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
            socket::commands::refresh_templates,
            socket::commands::replace_templates,
            socket::commands::read_focus,
            socket::commands::save_focus,
            socket::commands::home_dir,
            socket::commands::set_dialog_open,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Anchor the panel centered under the tray icon, on the icon's own monitor.
///
/// Driven by the tray event `rect` (authoritative for whichever display the
/// menu bar is on) rather than `tauri-plugin-positioner`'s `TrayCenter`, which
/// silently fails on extended monitors (plugins-workspace#724).
fn position_panel(window: &WebviewWindow, rect: &Rect, cursor: PhysicalPosition<f64>) {
    // The cursor is physical; use it to find the icon's monitor for scale and
    // work-area clamping.
    let monitor = window.monitor_from_point(cursor.x, cursor.y).ok().flatten();
    let scale = monitor.as_ref().map(|m| m.scale_factor()).unwrap_or(1.0);

    let icon_pos = rect.position.to_physical::<f64>(scale);
    let icon_size = rect.size.to_physical::<f64>(scale);
    let win = window.outer_size().unwrap_or_default();

    let mut x = icon_pos.x + icon_size.width / 2.0 - win.width as f64 / 2.0;
    let y = icon_pos.y + icon_size.height;

    // Clamp x so the panel never spills off an extended display.
    if let Some(m) = monitor {
        let min_x = m.position().x as f64;
        let max_x = min_x + m.size().width as f64 - win.width as f64;
        if max_x >= min_x {
            x = x.clamp(min_x, max_x);
        }
    }

    let _ = window.set_position(PhysicalPosition::new(x, y));
}

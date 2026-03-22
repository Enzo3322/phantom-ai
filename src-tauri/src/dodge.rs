#![allow(unexpected_cfgs, deprecated)]

use crate::state::AppState;
use tauri::Manager;

#[cfg(target_os = "macos")]
fn ease_out_cubic(t: f64) -> f64 {
    let t = t - 1.0;
    t * t * t + 1.0
}

#[cfg(target_os = "macos")]
async fn animate_window(window: &tauri::WebviewWindow, from_x: f64, to_x: f64, y: f64) {
    use std::time::Duration;
    use tauri::LogicalPosition;

    let steps = 18; // ~300ms at 60fps
    let frame_duration = Duration::from_millis(16);

    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let eased = ease_out_cubic(t);
        let x = from_x + (to_x - from_x) * eased;
        let _ = window.set_position(LogicalPosition::new(x, y));
        tokio::time::sleep(frame_duration).await;
    }
}

#[cfg(target_os = "macos")]
pub fn start_dodge_watcher(app: tauri::AppHandle) {
    use cocoa::base::id;
    use cocoa::foundation::NSPoint;
    use objc::{class, msg_send, sel, sel_impl};
    use std::time::{Duration, Instant};
    use tauri::Emitter;

    tauri::async_runtime::spawn(async move {
        let poll_interval = Duration::from_millis(200);
        let hover_threshold = Duration::from_secs(2);
        let cooldown = Duration::from_secs(1);
        let margin = 0.0;
        let width = 380.0;

        let mut hover_start: Option<Instant> = None;
        let mut last_dodge: Option<Instant> = None;
        let mut on_right = true;

        loop {
            tokio::time::sleep(poll_interval).await;

            let state = app.state::<AppState>();
            if !state.get_dodge_on_hover() {
                hover_start = None;
                continue;
            }

            if let Some(t) = last_dodge {
                if t.elapsed() < cooldown {
                    hover_start = None;
                    continue;
                }
            }

            let window = match app.get_webview_window("main") {
                Some(w) => w,
                None => {
                    hover_start = None;
                    continue;
                }
            };

            if !window.is_visible().unwrap_or(false) {
                hover_start = None;
                continue;
            }

            let scale = window.scale_factor().unwrap_or(1.0);

            let (win_x, win_y) = match window.outer_position() {
                Ok(pos) => (pos.x as f64 / scale, pos.y as f64 / scale),
                Err(_) => {
                    hover_start = None;
                    continue;
                }
            };

            let (win_w, win_h) = match window.outer_size() {
                Ok(size) => (size.width as f64 / scale, size.height as f64 / scale),
                Err(_) => {
                    hover_start = None;
                    continue;
                }
            };

            let (cursor_x, cursor_y, screen_width) = unsafe {
                let point: NSPoint = msg_send![class!(NSEvent), mouseLocation];
                let screen: id = msg_send![class!(NSScreen), mainScreen];
                let frame: cocoa::foundation::NSRect = msg_send![screen, frame];
                (point.x, frame.size.height - point.y, frame.size.width)
            };

            let inside = cursor_x >= win_x
                && cursor_x <= win_x + win_w
                && cursor_y >= win_y
                && cursor_y <= win_y + win_h;

            if inside {
                let start = hover_start.get_or_insert_with(Instant::now);
                if start.elapsed() >= hover_threshold {
                    let from_x = win_x;
                    let to_x = if on_right {
                        margin
                    } else {
                        screen_width - width - margin
                    };

                    animate_window(&window, from_x, to_x, win_y).await;
                    on_right = !on_right;

                    let _ = app.emit("dodge-move", ());
                    hover_start = None;
                    last_dodge = Some(Instant::now());
                }
            } else {
                hover_start = None;
            }
        }
    });
}

#[cfg(not(target_os = "macos"))]
pub fn start_dodge_watcher(_app: tauri::AppHandle) {}

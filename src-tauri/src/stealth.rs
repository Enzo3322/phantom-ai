#![allow(unexpected_cfgs, deprecated)]

use tauri::WebviewWindow;

/// NSWindowCollectionBehavior flags
#[cfg(target_os = "macos")]
const NS_WINDOW_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES: u64 = 1 << 0;
#[cfg(target_os = "macos")]
const NS_WINDOW_COLLECTION_BEHAVIOR_IGNORES_CYCLE: u64 = 1 << 4;
#[cfg(target_os = "macos")]
const NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY: u64 = 1 << 8;

/// Window levels
#[cfg(target_os = "macos")]
const NS_SCREEN_SAVER_WINDOW_LEVEL: i64 = 1000;
#[cfg(target_os = "macos")]
const NS_STATUS_WINDOW_LEVEL: i64 = 25;

#[cfg(target_os = "macos")]
pub fn apply_stealth(window: &WebviewWindow, stealth_enabled: bool) {
    use cocoa::base::{id, nil};
    use objc::{msg_send, sel, sel_impl};

    if let Ok(ns_window) = window.ns_window() {
        let ns_window = ns_window as id;
        unsafe {
            // 0 = invisible to screenshots, 1 = normal (visible)
            let sharing_type: u64 = if stealth_enabled { 0 } else { 1 };
            let _: () = msg_send![ns_window, setSharingType: sharing_type];

            // Transparent background so CSS handles it
            let _: () = msg_send![ns_window, setBackgroundColor: cocoa::appkit::NSColor::clearColor(nil)];
            let _: () = msg_send![ns_window, setOpaque: false];

            if stealth_enabled {
                // Elevate window level above proctoring overlays
                let _: () = msg_send![ns_window, setLevel: NS_STATUS_WINDOW_LEVEL];

                // Exclude from Expose, Spaces cycling, and fullscreen detection
                let behavior: u64 = NS_WINDOW_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES
                    | NS_WINDOW_COLLECTION_BEHAVIOR_IGNORES_CYCLE
                    | NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY;
                let _: () = msg_send![ns_window, setCollectionBehavior: behavior];
            }

            // Round corners via CALayer
            let content_view: id = msg_send![ns_window, contentView];
            if content_view != nil {
                let _: () = msg_send![content_view, setWantsLayer: true];
                let layer: id = msg_send![content_view, layer];
                if layer != nil {
                    let _: () = msg_send![layer, setCornerRadius: 14.0_f64];
                    let _: () = msg_send![layer, setMasksToBounds: true];
                }

                let superview: id = msg_send![content_view, superview];
                if superview != nil {
                    let _: () = msg_send![superview, setWantsLayer: true];
                    let sv_layer: id = msg_send![superview, layer];
                    if sv_layer != nil {
                        let _: () = msg_send![sv_layer, setCornerRadius: 14.0_f64];
                        let _: () = msg_send![sv_layer, setMasksToBounds: true];
                    }
                }
            }
        }
    }
}

#[cfg(target_os = "macos")]
pub fn set_window_level(window: &WebviewWindow, level: i64) {
    use cocoa::base::id;
    use objc::{msg_send, sel, sel_impl};

    if let Ok(ns_window) = window.ns_window() {
        unsafe {
            let _: () = msg_send![ns_window as id, setLevel: level];
        }
    }
}

#[cfg(target_os = "macos")]
pub fn set_passthrough_mode(window: &WebviewWindow, enabled: bool) {
    use cocoa::base::id;
    use objc::{msg_send, sel, sel_impl};

    if let Ok(ns_window) = window.ns_window() {
        unsafe {
            let _: () = msg_send![ns_window as id, setIgnoresMouseEvents: enabled];
        }
    }
}

#[cfg(target_os = "macos")]
pub fn set_stealth_for_all_windows(app: &tauri::AppHandle, stealth_enabled: bool) {
    use tauri::Manager;

    for label in &["config", "main", "welcome"] {
        if let Some(window) = app.get_webview_window(label) {
            apply_stealth(&window, stealth_enabled);
        }
    }
}

#[cfg(target_os = "macos")]
pub fn set_accessory_mode() {
    use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy};

    unsafe {
        let app = NSApp();
        app.setActivationPolicy_(
            NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory,
        );
    }
}

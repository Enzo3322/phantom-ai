use tauri::WebviewWindow;

#[cfg(target_os = "macos")]
pub fn apply_stealth(window: &WebviewWindow) {
    use cocoa::base::id;
    use objc::{msg_send, sel, sel_impl};

    if let Ok(ns_window) = window.ns_window() {
        let ns_window = ns_window as id;
        unsafe {
            // NSWindowSharingNone = 0 — invisible to screenshots and screen recording
            let _: () = msg_send![ns_window, setSharingType: 0u64];
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

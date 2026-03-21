use tauri::WebviewWindow;

#[cfg(target_os = "macos")]
pub fn apply_stealth(window: &WebviewWindow) {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};

    if let Ok(ns_window) = window.ns_window() {
        let ns_window = ns_window as id;
        unsafe {
            // NSWindowSharingNone = 0 — invisible to screenshots and screen recording
            let _: () = msg_send![ns_window, setSharingType: 0u64];

            // Force vibrancy to stay active even when window loses focus
            // Iterate contentView subviews to find NSVisualEffectView
            let content_view: id = msg_send![ns_window, contentView];
            if content_view != nil {
                let subviews: id = msg_send![content_view, subviews];
                let count: usize = msg_send![subviews, count];
                for i in 0..count {
                    let subview: id = msg_send![subviews, objectAtIndex: i];
                    let is_effect: bool = msg_send![subview, isKindOfClass: class!(NSVisualEffectView)];
                    if is_effect {
                        // NSVisualEffectStateActive = 1
                        let _: () = msg_send![subview, setState: 1i64];
                    }
                }
            }
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

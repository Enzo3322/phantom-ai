use tauri::WebviewWindow;

#[cfg(target_os = "macos")]
pub fn apply_stealth(window: &WebviewWindow) {
    use cocoa::base::{id, nil};
    use objc::{msg_send, sel, sel_impl};

    if let Ok(ns_window) = window.ns_window() {
        let ns_window = ns_window as id;
        unsafe {
            // Invisible to screenshots and screen recording
            let _: () = msg_send![ns_window, setSharingType: 0u64];

            // Transparent background so CSS handles it
            let _: () = msg_send![ns_window, setBackgroundColor: cocoa::appkit::NSColor::clearColor(nil)];
            let _: () = msg_send![ns_window, setOpaque: false];

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
pub fn set_accessory_mode() {
    use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy};

    unsafe {
        let app = NSApp();
        app.setActivationPolicy_(
            NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory,
        );
    }
}

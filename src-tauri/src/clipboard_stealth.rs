#![allow(unexpected_cfgs, deprecated)]

#[cfg(target_os = "macos")]
use std::time::Duration;

#[cfg(target_os = "macos")]
pub fn type_text_via_cgevents(text: &str, humanized: bool) {
    use core_graphics::event::{CGEvent, CGEventTapLocation};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    let source = match CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
        Ok(s) => s,
        Err(_) => return,
    };

    for ch in text.chars() {
        let mut buf = [0u16; 2];
        let encoded = ch.encode_utf16(&mut buf);

        let key_down = match CGEvent::new_keyboard_event(source.clone(), 0, true) {
            Ok(ev) => ev,
            Err(_) => continue,
        };
        key_down.set_string_from_utf16_unchecked(encoded);

        let key_up = match CGEvent::new_keyboard_event(source.clone(), 0, false) {
            Ok(ev) => ev,
            Err(_) => continue,
        };

        key_down.post(CGEventTapLocation::HID);
        key_up.post(CGEventTapLocation::HID);

        if humanized {
            let delay = if ch == ' ' || ch == '\n' {
                humanized_delay_word_boundary()
            } else {
                humanized_delay_char()
            };
            std::thread::sleep(delay);
        }
    }
}

#[cfg(target_os = "macos")]
fn humanized_delay_char() -> Duration {
    use rand::Rng;
    let ms = {
        let mut rng = rand::thread_rng();
        rng.gen_range(40..160)
    };
    Duration::from_millis(ms)
}

#[cfg(target_os = "macos")]
fn humanized_delay_word_boundary() -> Duration {
    use rand::Rng;
    let ms = {
        let mut rng = rand::thread_rng();
        rng.gen_range(100..400)
    };
    Duration::from_millis(ms)
}

#[cfg(target_os = "macos")]
pub fn ephemeral_paste(text: &str) {
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;
    use objc::{class, msg_send, sel, sel_impl};
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    unsafe {
        // Write to clipboard
        let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
        let _: () = msg_send![pasteboard, clearContents];

        let ns_string = NSString::alloc(nil).init_str(text);
        let string_type = NSString::alloc(nil).init_str("public.utf8-plain-text");
        let _: bool = msg_send![pasteboard, setString: ns_string forType: string_type];

        // Simulate Cmd+V
        let source = match CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
            Ok(s) => s,
            Err(_) => return,
        };

        // keycode 9 = V
        let v_down = match CGEvent::new_keyboard_event(source.clone(), 9, true) {
            Ok(ev) => ev,
            Err(_) => return,
        };
        v_down.set_flags(CGEventFlags::CGEventFlagCommand);

        let v_up = match CGEvent::new_keyboard_event(source.clone(), 9, false) {
            Ok(ev) => ev,
            Err(_) => return,
        };
        v_up.set_flags(CGEventFlags::CGEventFlagCommand);

        v_down.post(CGEventTapLocation::HID);
        v_up.post(CGEventTapLocation::HID);

        // Clear clipboard after short delay
        std::thread::sleep(Duration::from_millis(50));
        let _: () = msg_send![pasteboard, clearContents];
    }
}

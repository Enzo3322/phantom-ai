#![allow(unexpected_cfgs, deprecated)]

use crate::state::AppState;
use tauri::Manager;

#[cfg(target_os = "macos")]
pub fn disguise_process_name(name: &str) {
    use cocoa::base::id;
    use cocoa::foundation::NSString;
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let process_info: id = msg_send![class!(NSProcessInfo), processInfo];
        let ns_name = NSString::alloc(cocoa::base::nil).init_str(name);
        let _: () = msg_send![process_info, setProcessName: ns_name];
    }
}

#[cfg(target_os = "macos")]
pub fn scan_proctoring_software() -> Vec<DetectedProctor> {
    use cocoa::base::id;
    use cocoa::foundation::NSArray;
    use objc::{class, msg_send, sel, sel_impl};

    let known_proctors = [
        ("ProctorFree", &["com.proctorfree", "proctorfree"][..]),
        ("ProctorU", &["com.proctoru", "proctoru"][..]),
        ("Respondus LockDown", &["com.respondus", "lockdownbrowser"][..]),
        ("ExamSoft", &["com.examsoft", "examplify"][..]),
        ("Proctorio", &["com.proctorio", "proctorio"][..]),
        ("Honorlock", &["com.honorlock", "honorlock"][..]),
        ("ExamMonitor", &["com.exammonitor", "exammonitor"][..]),
    ];

    let mut detected = Vec::new();

    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let running_apps: id = msg_send![workspace, runningApplications];
        let count = NSArray::count(running_apps);

        for i in 0..count {
            let app: id = msg_send![running_apps, objectAtIndex: i];
            let bundle_id: id = msg_send![app, bundleIdentifier];
            let localized_name: id = msg_send![app, localizedName];

            let bundle_str = if bundle_id != cocoa::base::nil {
                let bytes: *const i8 = msg_send![bundle_id, UTF8String];
                if !bytes.is_null() {
                    std::ffi::CStr::from_ptr(bytes).to_string_lossy().to_lowercase()
                } else {
                    continue;
                }
            } else {
                continue;
            };

            let name_str = if localized_name != cocoa::base::nil {
                let bytes: *const i8 = msg_send![localized_name, UTF8String];
                if !bytes.is_null() {
                    std::ffi::CStr::from_ptr(bytes).to_string_lossy().to_lowercase()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            for (proctor_name, identifiers) in &known_proctors {
                let matched = identifiers.iter().any(|id| {
                    bundle_str.contains(id) || name_str.contains(id)
                });

                if matched {
                    detected.push(DetectedProctor {
                        name: proctor_name.to_string(),
                        bundle_id: bundle_str.clone(),
                        process_name: name_str.clone(),
                    });
                    break;
                }
            }
        }
    }

    detected
}

#[cfg(target_os = "macos")]
pub fn apply_process_stealth(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();
    let disguise_name = state.get_process_disguise_name();

    if !disguise_name.is_empty() {
        disguise_process_name(&disguise_name);
        eprintln!("[phantom] process disguised as: {}", disguise_name);
    }

    let detected = scan_proctoring_software();
    if !detected.is_empty() {
        for p in &detected {
            eprintln!("[phantom] detected proctoring: {} ({})", p.name, p.bundle_id);
        }
    }

    let json = serde_json::to_string(&detected).unwrap_or_default();
    *state.detected_proctors.lock().unwrap_or_else(|e| e.into_inner()) = detected;
    eprintln!("[phantom] proctoring scan complete: {}", json);
}

#[cfg(not(target_os = "macos"))]
pub fn apply_process_stealth(_app: &tauri::AppHandle) {}

#[cfg(not(target_os = "macos"))]
pub fn scan_proctoring_software() -> Vec<DetectedProctor> {
    Vec::new()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DetectedProctor {
    pub name: String,
    pub bundle_id: String,
    pub process_name: String,
}

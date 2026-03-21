use base64::{engine::general_purpose::STANDARD, Engine};
use std::process::Command;

pub fn capture_screen() -> Result<String, String> {
    let tmp_path = std::env::temp_dir().join("phantom_capture.png");
    let tmp_str = tmp_path.to_string_lossy().to_string();

    // Use macOS native screencapture tool — stable and doesn't crash
    let output = Command::new("screencapture")
        .args(["-x", "-C", "-t", "png", &tmp_str])
        .output()
        .map_err(|e| format!("Failed to run screencapture: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("screencapture failed: {stderr}"));
    }

    let bytes =
        std::fs::read(&tmp_path).map_err(|e| format!("Failed to read screenshot file: {e}"))?;

    let _ = std::fs::remove_file(&tmp_path);

    if bytes.is_empty() {
        return Err("Screenshot is empty. Check screen recording permission.".to_string());
    }

    Ok(STANDARD.encode(bytes))
}

#[cfg(target_os = "macos")]
pub fn check_screen_permission() -> bool {
    extern "C" {
        fn CGPreflightScreenCaptureAccess() -> bool;
        fn CGRequestScreenCaptureAccess() -> bool;
    }
    unsafe {
        if !CGPreflightScreenCaptureAccess() {
            return CGRequestScreenCaptureAccess();
        }
        true
    }
}

#[cfg(not(target_os = "macos"))]
pub fn check_screen_permission() -> bool {
    true
}

use base64::{engine::general_purpose::STANDARD, Engine};
use image::ImageFormat;
use std::io::Cursor;
use xcap::Monitor;

pub fn capture_screen() -> Result<String, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to list monitors: {e}"))?;
    let primary = monitors.into_iter().next().ok_or("No monitor found")?;

    let image = primary
        .capture_image()
        .map_err(|e| format!("Failed to capture screen: {e}"))?;

    let mut buffer = Cursor::new(Vec::new());
    image
        .write_to(&mut buffer, ImageFormat::Png)
        .map_err(|e| format!("Failed to encode image: {e}"))?;

    Ok(STANDARD.encode(buffer.into_inner()))
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

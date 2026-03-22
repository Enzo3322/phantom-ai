use std::process::Command;

const OCR_SCRIPT: &str = r#"
import Vision
import AppKit
let url = URL(fileURLWithPath: CommandLine.arguments[1])
guard let image = NSImage(contentsOf: url),
      let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil)
else { exit(1) }
let request = VNRecognizeTextRequest()
request.recognitionLevel = .accurate
try VNImageRequestHandler(cgImage: cgImage).perform([request])
let text = (request.results ?? []).compactMap { $0.topCandidates(1).first?.string }.joined(separator: "\n")
print(text)
"#;

pub fn capture_screen() -> Result<String, String> {
    let tmp_path = std::env::temp_dir().join("phantom_capture.jpg");
    let tmp_str = tmp_path.to_string_lossy().to_string();

    let output = Command::new("screencapture")
        .args(["-x", "-C", "-t", "jpg", &tmp_str])
        .output()
        .map_err(|e| format!("Failed to run screencapture: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("screencapture failed: {stderr}"));
    }

    let ocr_result = Command::new("swift")
        .args(["-e", OCR_SCRIPT, &tmp_str])
        .output()
        .map_err(|e| format!("Failed to run OCR: {e}"));

    let _ = std::fs::remove_file(&tmp_path);

    let ocr_output = ocr_result?;

    if !ocr_output.status.success() {
        let stderr = String::from_utf8_lossy(&ocr_output.stderr);
        return Err(format!("OCR failed: {stderr}"));
    }

    let text = String::from_utf8_lossy(&ocr_output.stdout).trim().to_string();

    if text.is_empty() {
        return Err("OCR extracted no text from screenshot.".to_string());
    }

    eprintln!("[phantom] capture: OCR extracted {} chars", text.len());

    Ok(text)
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

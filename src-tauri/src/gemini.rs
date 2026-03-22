use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Part {
    Text {
        text: String,
    },
    InlineData {
        inline_data: InlineData,
    },
}

#[derive(Serialize)]
struct InlineData {
    mime_type: String,
    data: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    error: Option<GeminiError>,
}

#[derive(Deserialize)]
struct GeminiError {
    message: String,
}

#[derive(Deserialize)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Deserialize)]
struct CandidateContent {
    #[serde(default)]
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    #[serde(default)]
    text: String,
}

pub async fn analyze_screenshot(
    api_key: &str,
    model: &str,
    base64_image: &str,
    prompt: &str,
    response_language: &str,
) -> Result<String, String> {
    let lang_instruction = match response_language {
        "auto" | "" => String::new(),
        lang => format!("\n\nIMPORTANT: You MUST respond in {lang}."),
    };
    let full_prompt = format!("{prompt}{lang_instruction}");

    let client = Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let request = GeminiRequest {
        contents: vec![Content {
            parts: vec![
                Part::InlineData {
                    inline_data: InlineData {
                        mime_type: "image/jpeg".to_string(),
                        data: base64_image.to_string(),
                    },
                },
                Part::Text {
                    text: full_prompt,
                },
            ],
        }],
    };

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    let status = response.status();
    let raw = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    eprintln!("[phantom] gemini status={status} body_len={}", raw.len());
    eprintln!("[phantom] gemini body: {}", &raw[..raw.len().min(500)]);

    if !status.is_success() {
        if let Ok(parsed) = serde_json::from_str::<GeminiResponse>(&raw) {
            if let Some(err) = parsed.error {
                return Err(format!("Gemini API error ({}): {}", status, err.message));
            }
        }
        return Err(format!("Gemini API error ({}): {}", status, &raw[..raw.len().min(300)]));
    }

    let body: GeminiResponse = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    let result = body
        .candidates
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|p| p.text)
        .unwrap_or_default();

    eprintln!("[phantom] gemini result: {}", &result[..result.len().min(200)]);

    if result.is_empty() {
        return Err("Empty response from Gemini".to_string());
    }

    Ok(result)
}

pub async fn send_to_gemini(
    api_key: &str,
    model: &str,
    text: &str,
    prompt: &str,
    response_language: &str,
) -> Result<String, String> {
    let lang_instruction = match response_language {
        "auto" | "" => String::new(),
        lang => format!("\n\nIMPORTANT: You MUST respond in {lang}."),
    };
    let full_prompt = format!("{prompt}{lang_instruction}\n\nTranscription:\n{text}");

    let client = Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let request = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part::Text {
                text: full_prompt,
            }],
        }],
    };

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    let status = response.status();
    let raw = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    if !status.is_success() {
        return Err(format!(
            "Gemini API error ({}): {}",
            status,
            &raw[..raw.len().min(300)]
        ));
    }

    let body: GeminiResponse = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    body.candidates
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|p| p.text)
        .ok_or_else(|| "Empty response from Gemini".to_string())
}

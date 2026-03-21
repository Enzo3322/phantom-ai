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
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: String,
}

pub async fn analyze_screenshot(
    api_key: &str,
    model: &str,
    base64_image: &str,
    prompt: &str,
) -> Result<String, String> {
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
                        mime_type: "image/png".to_string(),
                        data: base64_image.to_string(),
                    },
                },
                Part::Text {
                    text: prompt.to_string(),
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

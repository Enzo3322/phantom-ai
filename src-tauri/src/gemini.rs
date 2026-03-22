use serde::{Deserialize, Serialize};
use crate::network_stealth;

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
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    error: Option<GeminiError>,
    #[serde(default, rename = "usageMetadata")]
    usage_metadata: Option<UsageMetadata>,
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

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct UsageMetadata {
    #[serde(default)]
    prompt_token_count: u32,
    #[serde(default)]
    candidates_token_count: u32,
}

#[derive(Clone, Debug, Default)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

fn extract_usage(metadata: Option<UsageMetadata>) -> TokenUsage {
    match metadata {
        Some(m) => TokenUsage {
            input_tokens: m.prompt_token_count,
            output_tokens: m.candidates_token_count,
        },
        None => TokenUsage::default(),
    }
}

async fn call_gemini(
    api_key: &str,
    model: &str,
    parts: Vec<Part>,
    spoof_ua: bool,
    jitter: bool,
    proxy_url: Option<&str>,
) -> Result<(String, TokenUsage), String> {
    if jitter {
        network_stealth::apply_jitter().await;
    }

    let client = if spoof_ua {
        network_stealth::build_stealth_client(proxy_url)?
    } else {
        reqwest::Client::new()
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let request = GeminiRequest {
        contents: vec![Content { parts }],
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

    let usage = extract_usage(body.usage_metadata);

    let text = body
        .candidates
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|p| p.text)
        .ok_or_else(|| "Empty response from Gemini".to_string())?;

    Ok((text, usage))
}

pub async fn send_to_gemini(
    api_key: &str,
    model: &str,
    text: &str,
    prompt: &str,
    response_language: &str,
    spoof_ua: bool,
    jitter: bool,
    proxy_url: Option<&str>,
) -> Result<(String, TokenUsage), String> {
    let lang_instruction = match response_language {
        "auto" | "" => String::new(),
        lang => format!("\n\nIMPORTANT: You MUST respond in {lang}."),
    };
    let full_prompt = format!("{prompt}{lang_instruction}\n\nTranscription:\n{text}");

    let parts = vec![Part::Text { text: full_prompt }];

    call_gemini(api_key, model, parts, spoof_ua, jitter, proxy_url).await
}


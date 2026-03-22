use crate::vad::{self, Vad};

pub fn trim_trailing_silence(audio: &[f32]) -> &[f32] {
    let frame = vad::SAMPLE_RATE / 50; // 20ms frames
    let mut last_speech_end = audio.len();

    for start in (0..audio.len()).rev().step_by(frame) {
        let end = (start + frame).min(audio.len());
        let chunk = &audio[start..end];
        let rms = Vad::frame_rms(chunk);
        if rms > vad::MIN_SPEECH_THRESHOLD * 2.0 {
            last_speech_end = (end + vad::SAMPLE_RATE / 10).min(audio.len());
            break;
        }
    }

    &audio[..last_speech_end]
}

pub fn is_duplicate_segment(text: &str, previous: &[String]) -> bool {
    let lower = text.to_lowercase();

    previous.iter().any(|prev| {
        let prev_lower = prev.to_lowercase();

        if prev_lower == lower {
            return true;
        }

        if let Some(prefix) = floor_char_boundary(&prev_lower, prev_lower.len() * 2 / 3) {
            if prev_lower.len() > 10 && lower.contains(prefix) {
                return true;
            }
        }
        if let Some(prefix) = floor_char_boundary(&lower, lower.len() * 2 / 3) {
            if lower.len() > 10 && prev_lower.contains(prefix) {
                return true;
            }
        }

        false
    })
}

fn floor_char_boundary(s: &str, index: usize) -> Option<&str> {
    if index >= s.len() {
        return Some(s);
    }
    let mut i = index;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    if i == 0 { None } else { Some(&s[..i]) }
}

pub fn is_hallucination(text: &str) -> bool {
    let lower = text.to_lowercase();
    let trimmed = lower.trim();

    if trimmed.len() < 2 {
        return true;
    }

    if trimmed.chars().all(|c| !c.is_alphanumeric()) {
        return true;
    }

    let exact = [
        "thank you",
        "thank you for watching",
        "thanks for watching",
        "thank you so much",
        "thanks for listening",
        "subscribe",
        "like and subscribe",
        "please subscribe",
        "see you next time",
        "see you in the next video",
        "see you later",
        "bye bye",
        "bye",
        "goodbye",
        "good night",
        "the end",
        "you",
        "so",
        "oh",
        "hmm",
        "um",
        "uh",
        "ah",
        "huh",
        "what",
        "what?",
        "why",
        "how",
        "yes",
        "no",
        "ok",
        "okay",
        "music",
        "applause",
        "silence",
        "laughter",
        "obrigado",
        "obrigada",
        "tchau",
        "valeu",
        "legendas pela comunidade",
        "sim",
        "não",
        "tá",
        "né",
        "bom",
        "bem",
        "então",
    ];

    if exact.contains(&trimmed) {
        return true;
    }

    let patterns = [
        "subtitles by",
        "amara.org",
        "www.",
        "http",
        "translated by",
        "captions by",
        "copyright",
        "all rights reserved",
        "please like",
        "don't forget to",
        "hit the bell",
        "follow me on",
        "check out",
    ];

    for pattern in &patterns {
        if trimmed.starts_with(pattern) || trimmed.contains(pattern) {
            return true;
        }
    }

    let question_hallucinations = [
        "what's the reason",
        "what is the reason",
        "what do you think",
        "what do you mean",
        "what is this",
        "what is that",
        "what are you doing",
        "what happened",
        "how are you",
        "how do you do",
        "who are you",
        "who is this",
        "where are you",
        "where is this",
        "why is that",
        "why not",
        "is that so",
        "is it",
        "is that right",
        "do you know",
        "did you know",
        "can you",
        "could you",
        "really",
        "right",
    ];

    if question_hallucinations.contains(&trimmed) {
        return true;
    }

    let words: Vec<&str> = trimmed.split_whitespace().collect();

    if words.len() <= 2 && trimmed.len() < 10 {
        return true;
    }

    // Single word repeated: "you you you you"
    if words.len() >= 3 {
        let first = words[0];
        if words.iter().filter(|&&w| w == first).count() >= words.len() * 2 / 3 {
            return true;
        }
    }

    // N-gram repeated 2+ times indicates hallucination loop
    if words.len() >= 4 {
        for n in 2..=4 {
            if words.len() < n * 2 {
                continue;
            }
            for start in 0..words.len().saturating_sub(n) {
                let ngram = words[start..start + n].join(" ");
                if trimmed.matches(&ngram).count() >= 2 {
                    return true;
                }
            }
        }
    }

    // Foreign script detection (Arabic, Cyrillic, CJK) — likely hallucination
    let alpha_chars: Vec<char> = trimmed.chars().filter(|c| c.is_alphabetic()).collect();
    if alpha_chars.len() > 5 {
        let non_latin = alpha_chars
            .iter()
            .filter(|c| {
                let cp = **c as u32;
                cp > 0x024F
            })
            .count();
        if non_latin > alpha_chars.len() / 2 {
            return true;
        }
    }

    false
}

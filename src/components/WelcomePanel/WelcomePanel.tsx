import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./WelcomePanel.css";

export function WelcomePanel() {
  const [apiKey, setApiKey] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const handleGetStarted = async () => {
    if (!apiKey.trim()) {
      setError("Please enter your Gemini API key");
      return;
    }

    setLoading(true);
    setError("");
    try {
      await invoke("complete_onboarding", { apiKey: apiKey.trim() });
    } catch (e) {
      setError(String(e));
      setLoading(false);
    }
  };

  const handleOpenApiStudio = () => {
    invoke("open_external_url", { url: "https://aistudio.google.com/apikey" });
  };

  return (
    <div className="welcome-panel">
      <div className="welcome-titlebar" data-tauri-drag-region />

      <div className="welcome-content">
        <div className="welcome-header">
          <svg
            className="welcome-icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
          >
            <path
              d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"
              strokeLinecap="round"
            />
            <path d="M9 9h.01M15 9h.01" strokeLinecap="round" />
            <path
              d="M8 13c1 2 3 3 4 3s3-1 4-3"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
          <h1 className="welcome-title">Welcome to Phantom</h1>
          <p className="welcome-subtitle">
            Your AI assistant that sees your screen and hears your audio.
            Invisible, always ready.
          </p>
        </div>

        <div className="welcome-step">
          <span className="welcome-step-label">API Key</span>
          <div className="welcome-input-row">
            <input
              type="password"
              value={apiKey}
              onChange={(e) => {
                setApiKey(e.target.value);
                setError("");
              }}
              placeholder="Enter your Gemini API key"
              spellCheck={false}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleGetStarted();
              }}
            />
          </div>
          <span className="welcome-api-link" onClick={handleOpenApiStudio}>
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
              <polyline points="15 3 21 3 21 9" />
              <line x1="10" y1="14" x2="21" y2="3" />
            </svg>
            Get a free API key from Google AI Studio
          </span>
        </div>

        <div className="welcome-step">
          <span className="welcome-step-label">How it works</span>
          <div className="welcome-shortcuts">
            <div className="welcome-shortcut-card">
              <div className="welcome-shortcut-icon capture">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <rect x="3" y="3" width="18" height="18" rx="2" />
                  <circle cx="12" cy="12" r="3" />
                </svg>
              </div>
              <div className="welcome-shortcut-info">
                <span className="welcome-shortcut-name">Screenshot & Analyze</span>
                <span className="welcome-shortcut-desc">Capture your screen and get AI analysis</span>
              </div>
              <kbd>⌘ ⇧ S</kbd>
            </div>

            <div className="welcome-shortcut-card">
              <div className="welcome-shortcut-icon record">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
                  <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
                  <line x1="12" y1="19" x2="12" y2="23" />
                </svg>
              </div>
              <div className="welcome-shortcut-info">
                <span className="welcome-shortcut-name">Record & Transcribe</span>
                <span className="welcome-shortcut-desc">Transcribe audio in real-time with AI</span>
              </div>
              <kbd>⌘ ⇧ M</kbd>
            </div>

            <div className="welcome-shortcut-card">
              <div className="welcome-shortcut-icon settings">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <circle cx="12" cy="12" r="3" />
                  <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
                </svg>
              </div>
              <div className="welcome-shortcut-info">
                <span className="welcome-shortcut-name">Settings</span>
                <span className="welcome-shortcut-desc">Configure model, prompts, and audio</span>
              </div>
              <kbd>⌘ ⇧ C</kbd>
            </div>
          </div>
        </div>

        {error && <p className="welcome-error">{error}</p>}

        <button
          className="welcome-cta"
          onClick={handleGetStarted}
          disabled={loading}
        >
          {loading ? "Setting up..." : "Get Started"}
        </button>
      </div>
    </div>
  );
}

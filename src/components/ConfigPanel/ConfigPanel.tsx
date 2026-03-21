import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { GlassContainer } from "../shared/GlassContainer";
import { useConfig } from "../../hooks/useConfig";
import "./ConfigPanel.css";

const MODELS = [
  { value: "gemini-2.0-flash", label: "Gemini 2.0 Flash" },
  { value: "gemini-2.5-flash", label: "Gemini 2.5 Flash" },
  { value: "gemini-2.5-pro", label: "Gemini 2.5 Pro" },
  { value: "gemini-3.1-pro-preview", label: "Gemini 3.1" },
];

export function ConfigPanel() {
  const { config, save, saving, saved } = useConfig();
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState("gemini-2.0-flash");
  const [prompt, setPrompt] = useState("");
  const [showKey, setShowKey] = useState(false);

  useEffect(() => {
    setApiKey(config.api_key);
    setModel(config.model);
    setPrompt(config.prompt);
  }, [config]);

  const handleSave = () => {
    save({ api_key: apiKey, model, prompt });
  };

  const handleClose = () => {
    getCurrentWindow().hide();
  };

  return (
    <GlassContainer>
      <div className="titlebar" data-tauri-drag-region>
        <span className="titlebar-title">Phantom Settings</span>
        <button className="close-btn" onClick={handleClose}>
          <svg width="10" height="10" viewBox="0 0 10 10">
            <path
              d="M1 1L9 9M9 1L1 9"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
            />
          </svg>
        </button>
      </div>

      <div className="config-content">
        <div className="field">
          <label>API Key</label>
          <div className="input-row">
            <input
              type={showKey ? "text" : "password"}
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="Enter your Gemini API key"
              spellCheck={false}
            />
            <button
              className="toggle-btn"
              onClick={() => setShowKey(!showKey)}
            >
              {showKey ? "Hide" : "Show"}
            </button>
          </div>
        </div>

        <div className="field">
          <label>Model</label>
          <select value={model} onChange={(e) => setModel(e.target.value)}>
            {MODELS.map((m) => (
              <option key={m.value} value={m.value}>
                {m.label}
              </option>
            ))}
          </select>
        </div>

        <div className="field">
          <label>Prompt</label>
          <textarea
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            rows={4}
            placeholder="Instructions sent with each screenshot"
            spellCheck={false}
          />
        </div>

        <div className="field">
          <label>Shortcuts</label>
          <div className="shortcuts">
            <div className="shortcut-row">
              <span className="shortcut-label">Capture & Analyze</span>
              <kbd>⌘ ⇧ S</kbd>
            </div>
            <div className="shortcut-row">
              <span className="shortcut-label">Toggle Settings</span>
              <kbd>⌘ ⇧ C</kbd>
            </div>
            <div className="shortcut-row">
              <span className="shortcut-label">Toggle Response</span>
              <kbd>⌘ ⇧ A</kbd>
            </div>
          </div>
        </div>

        <button
          className="save-btn"
          onClick={handleSave}
          disabled={saving}
        >
          {saved ? "Saved ✓" : saving ? "Saving..." : "Save"}
        </button>
      </div>
    </GlassContainer>
  );
}

import { useState, useEffect, useRef, useCallback } from "react";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { GlassContainer } from "../shared/GlassContainer";
import { useConfig } from "../../hooks/useConfig";
import "./ConfigPanel.css";

const WIDTH = 460;

const QUICK_PROMPTS = [
  {
    label: "Direct Answer",
    prompt:
      "Look at the screen and identify any question or quiz. Reply ONLY with the correct answer, nothing else. No explanation, no reasoning.",
  },
  {
    label: "Answer + Explanation",
    prompt:
      "Look at the screen and identify any question or quiz. Reply with the correct answer followed by a brief explanation of why it is correct.",
  },
  {
    label: "Step by Step",
    prompt:
      "Look at the screen and identify any question or quiz. Solve it step by step, showing your reasoning clearly, then state the final answer.",
  },
  {
    label: "Code Help",
    prompt:
      "Analyze the code visible on screen. Identify any errors, answer any questions about it, or suggest improvements. Be concise.",
  },
];

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
  const [opacity, setOpacity] = useState(0.85);
  const [stealthMode, setStealthMode] = useState(true);
  const [showKey, setShowKey] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);

  const resizeToFit = useCallback(async () => {
    if (!panelRef.current) return;
    await new Promise((r) => requestAnimationFrame(r));
    const height = panelRef.current.scrollHeight;
    await getCurrentWindow().setSize(new LogicalSize(WIDTH, height));
  }, []);

  useEffect(() => {
    resizeToFit();
  }, [resizeToFit]);

  useEffect(() => {
    setApiKey(config.api_key);
    setModel(config.model);
    setPrompt(config.prompt);
    setOpacity(config.opacity);
    setStealthMode(config.stealth_mode);
  }, [config]);

  const handleOpacityChange = (val: number) => {
    setOpacity(val);
    document.documentElement.style.setProperty("--bg-opacity", String(val));
  };

  const handleSave = () => {
    save({ api_key: apiKey, model, prompt, opacity, stealth_mode: stealthMode });
  };

  const handleClose = () => {
    getCurrentWindow().hide();
  };

  return (
    <GlassContainer ref={panelRef}>
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
          <div className="quick-prompts">
            {QUICK_PROMPTS.map((qp) => (
              <button
                key={qp.label}
                className={`quick-prompt-btn ${prompt === qp.prompt ? "active" : ""}`}
                onClick={() => setPrompt(qp.prompt)}
              >
                {qp.label}
              </button>
            ))}
          </div>
          <textarea
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            rows={3}
            placeholder="Instructions sent with each screenshot"
            spellCheck={false}
          />
        </div>

        <div className="field">
          <label>
            Opacity
            <span className="opacity-value">{Math.round(opacity * 100)}%</span>
          </label>
          <input
            type="range"
            min="0.1"
            max="1"
            step="0.05"
            value={opacity}
            onChange={(e) => handleOpacityChange(Number(e.target.value))}
            className="opacity-slider"
          />
        </div>

        <div className="field">
          <div className="stealth-row">
            <div className="stealth-info">
              <label>Stealth Mode</label>
              <span className="stealth-desc">Hide window from screenshots and recordings</span>
            </div>
            <button
              className={`stealth-toggle ${stealthMode ? "active" : ""}`}
              onClick={() => setStealthMode(!stealthMode)}
            >
              <span className="stealth-toggle-knob" />
            </button>
          </div>
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
          {saved ? "Saved" : saving ? "Saving..." : "Save"}
        </button>
      </div>
    </GlassContainer>
  );
}

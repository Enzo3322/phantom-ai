import { getCurrentWindow } from "@tauri-apps/api/window";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useState } from "react";
import Markdown from "react-markdown";
import { GlassContainer } from "../shared/GlassContainer";
import { LoadingSpinner } from "../shared/LoadingSpinner";
import { useGemini } from "../../hooks/useGemini";
import "./ResponsePanel.css";

export function ResponsePanel() {
  const { response, loading, error } = useGemini();
  const [copied, setCopied] = useState(false);

  const handleClose = () => {
    getCurrentWindow().hide();
  };

  const handleCopy = async () => {
    if (response) {
      try {
        await writeText(response);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      } catch {
        // clipboard not available
      }
    }
  };

  return (
    <GlassContainer>
      <div className="titlebar" data-tauri-drag-region>
        <span className="titlebar-title">Phantom</span>
        <div className="titlebar-actions">
          {response && (
            <button className="action-btn" onClick={handleCopy}>
              {copied ? "Copied ✓" : "Copy"}
            </button>
          )}
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
      </div>

      <div className="response-content">
        {loading && <LoadingSpinner />}

        {error && !loading && (
          <div className="error-message">
            <span className="error-icon">!</span>
            <p>{error}</p>
          </div>
        )}

        {response && !loading && (
          <div className="markdown-body">
            <Markdown>{response}</Markdown>
          </div>
        )}

        {!response && !loading && !error && (
          <div className="empty-state">
            <p className="empty-title">No response yet</p>
            <p className="empty-hint">
              Press <kbd>⌘ ⇧ S</kbd> to capture and analyze
            </p>
          </div>
        )}
      </div>
    </GlassContainer>
  );
}

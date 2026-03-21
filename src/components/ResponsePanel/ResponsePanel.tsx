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
        {response && (
          <button className="action-btn" onClick={handleCopy}>
            {copied ? "Copied" : "Copy"}
          </button>
        )}
      </div>

      <div className="response-content">
        {loading && <LoadingSpinner />}

        {error && !loading && (
          <div className="error-message">
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
            <p className="empty-hint">
              <kbd>⌘ ⇧ S</kbd> to capture
            </p>
          </div>
        )}
      </div>
    </GlassContainer>
  );
}

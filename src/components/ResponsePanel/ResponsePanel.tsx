import { getCurrentWindow, LogicalSize, LogicalPosition } from "@tauri-apps/api/window";
import { currentMonitor } from "@tauri-apps/api/window";
import { useRef, useEffect, useCallback } from "react";
import Markdown from "react-markdown";
import { LoadingSpinner } from "../shared/LoadingSpinner";
import { useGemini } from "../../hooks/useGemini";
import "./ResponsePanel.css";

const MIN_HEIGHT = 48;
const MAX_HEIGHT = 600;
const WIDTH = 380;
const MARGIN = 16;

export function ResponsePanel() {
  const { response, loading, error } = useGemini();
  const contentRef = useRef<HTMLDivElement>(null);

  const resizeToFit = useCallback(async () => {
    if (!contentRef.current) return;

    // Wait a frame for content to render
    await new Promise((r) => requestAnimationFrame(r));

    const contentHeight = contentRef.current.scrollHeight;
    const totalHeight = Math.min(
      Math.max(contentHeight, MIN_HEIGHT),
      MAX_HEIGHT
    );

    const win = getCurrentWindow();
    await win.setSize(new LogicalSize(WIDTH, totalHeight));

    // Position top-right
    const monitor = await currentMonitor();
    if (monitor) {
      const screenWidth = monitor.size.width / monitor.scaleFactor;
      const x = screenWidth - WIDTH - MARGIN;
      await win.setPosition(new LogicalPosition(x, MARGIN));
    }
  }, []);

  useEffect(() => {
    resizeToFit();
  }, [response, loading, error, resizeToFit]);

  return (
    <div className="response-panel" data-tauri-drag-region>
      <div className="response-content" ref={contentRef}>
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
    </div>
  );
}

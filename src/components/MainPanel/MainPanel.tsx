import { useRef, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  getCurrentWindow,
  LogicalSize,
  LogicalPosition,
  currentMonitor,
} from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import Markdown from "react-markdown";
import { LoadingSpinner } from "../shared/LoadingSpinner";
import { useGemini } from "../../hooks/useGemini";
import { useRecording } from "../../hooks/useRecording";
import { useTranscript } from "../../hooks/useTranscript";
import "./MainPanel.css";

const WIDTH = 380;
const MIN_RESPONSE_HEIGHT = 120;
const TITLEBAR_HEIGHT = 40;
const PADDING = 24;
const MARGIN = 0;

type Mode = "idle" | "response" | "recording" | "processing" | "error";

export function MainPanel() {
  const {
    response,
    loading: geminiLoading,
    error: geminiError,
    source: geminiSource,
    model,
    clearResponse,
    goBack,
    goForward,
    canGoBack,
    canGoForward,
    historyCount,
    historyIndex,
  } = useGemini();
  const { isRecording, toggleRecording, stopRecording } = useRecording();
  const {
    transcript,
    preview,
    isComplete,
    error: transcriptionError,
    clearTranscript,
  } = useTranscript();
  const [dismissedError, setDismissedError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const sideRef = useRef<"right" | "left">("right");
  const scrollRef = useRef<HTMLDivElement>(null);
  const responseRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const screenRef = useRef({ width: 1440, height: 900 });

  const error = geminiError || transcriptionError;
  const activeError = error && error !== dismissedError ? error : null;

  const mode: Mode = (() => {
    if (activeError) return "error";
    if (isRecording) return "recording";
    if (response) return "response";
    if (geminiLoading) return "processing";
    return "idle";
  })();

  // Cache screen dimensions once on mount
  useEffect(() => {
    currentMonitor().then((monitor) => {
      if (monitor) {
        screenRef.current = {
          width: monitor.size.width / monitor.scaleFactor,
          height: monitor.size.height / monitor.scaleFactor,
        };
      }
    });
  }, []);

  // Resize and show/hide based on mode
  useEffect(() => {
    const { width: screenWidth, height: screenHeight } = screenRef.current;
    const win = getCurrentWindow();
    const maxHeight = screenHeight - MARGIN * 2;

    if (mode === "idle") {
      win.hide();
      return;
    }

    let height: number;
    if (mode === "recording") {
      height = 400;
    } else if (mode === "response") {
      const contentHeight = contentRef.current?.scrollHeight ?? 0;
      const computed = TITLEBAR_HEIGHT + contentHeight + PADDING;
      height = Math.max(MIN_RESPONSE_HEIGHT, Math.min(computed, maxHeight));
    } else if (mode === "processing") {
      height = 120;
    } else {
      height = 120;
    }

    const x = sideRef.current === "right"
      ? screenWidth - WIDTH - MARGIN
      : MARGIN;

    win.setSize(new LogicalSize(WIDTH, height));
    win.setPosition(new LogicalPosition(x, MARGIN));
    win.show();
  }, [mode, response, transcript]);

  // Clear stale Gemini response when a new recording starts
  useEffect(() => {
    if (isRecording) {
      clearResponse();
    }
  }, [isRecording, clearResponse]);

  // Clear transcript when a screenshot processing starts (not from transcription)
  useEffect(() => {
    if (geminiLoading && geminiSource === "screenshot") {
      clearTranscript();
    }
  }, [geminiLoading, geminiSource, clearTranscript]);

  // Auto-send to Gemini ONLY after transcription is fully complete
  useEffect(() => {
    if (isComplete && !isRecording && transcript.trim()) {
      invoke("send_transcription_to_gemini", {
        text: transcript,
        prompt: "Analyze the following audio transcription and provide a helpful response.",
      }).catch((e) => console.error("Auto-send to Gemini failed:", e));
    }
  }, [isComplete, isRecording, transcript]);

  // Auto-scroll transcript
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [transcript, preview]);

  // Auto-scroll response
  useEffect(() => {
    if (responseRef.current) {
      responseRef.current.scrollTop = responseRef.current.scrollHeight;
    }
  }, [response]);

  // Sync side ref when Rust backend animates the window
  useEffect(() => {
    const unlisten = listen("dodge-move", () => {
      sideRef.current = sideRef.current === "right" ? "left" : "right";
    });
    return () => { unlisten.then((f) => f()); };
  }, []);

  const handleToggleRecording = async () => {
    try {
      await toggleRecording();
    } catch (e) {
      console.error("Toggle recording failed:", e);
    }
  };

  const handleClear = async () => {
    if (isRecording) {
      await stopRecording();
    }
    clearTranscript();
    clearResponse();
  };

  // --- IDLE (window is hidden) ---
  if (mode === "idle") {
    return null;
  }

  // --- ERROR ---
  if (mode === "error") {
    return (
      <div className="main-panel">
        <div className="main-titlebar" data-tauri-drag-region>
          <div className="main-title-left">
            <svg className="main-title-icon error-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10" />
              <line x1="15" y1="9" x2="9" y2="15" />
              <line x1="9" y1="9" x2="15" y2="15" />
            </svg>
            <span className="main-title">Error</span>
          </div>
          <button className="main-close-btn" onClick={() => {
            setDismissedError(activeError);
          }}>
            <svg width="10" height="10" viewBox="0 0 10 10">
              <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
          </button>
        </div>
        <div className="main-body">
          <div className="main-error">
            <p>{activeError}</p>
          </div>
        </div>
      </div>
    );
  }

  // --- RECORDING ---
  if (mode === "recording") {
    return (
      <div className="main-panel">
        <div className="main-titlebar" data-tauri-drag-region>
          <div className="main-title-left">
            <span className="rec-dot" />
            <span className="main-title">Recording...</span>
          </div>
          <div className="main-title-right">
            <button
              className="rec-toggle-btn recording"
              onClick={handleToggleRecording}
            >
              <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor"><rect x="4" y="4" width="16" height="16" rx="2" /></svg>
              Stop
            </button>
          </div>
        </div>
        <div className="main-body" ref={scrollRef}>
          {transcript || preview ? (
            <div className="main-transcript">
              {transcript}
              {preview && (
                <>
                  {transcript && "\n"}
                  <span className="transcript-preview">{preview}</span>
                </>
              )}
            </div>
          ) : (
            <div className="main-listening">
              <LoadingSpinner />
              <span>Listening...</span>
            </div>
          )}
        </div>
      </div>
    );
  }

  // --- PROCESSING ---
  if (mode === "processing") {
    return (
      <div className="main-panel">
        <div className="main-titlebar" data-tauri-drag-region>
          <div className="main-title-left">
            <svg
              className="main-title-icon brain-icon"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M9.5 2a3.5 3.5 0 0 0-3.4 4.4A3.5 3.5 0 0 0 4 10a3.5 3.5 0 0 0 1.8 3.1A3.5 3.5 0 0 0 7 17a3.5 3.5 0 0 0 3.5 3.5c.8 0 1.5-.2 2.1-.6" />
              <path d="M14.5 2a3.5 3.5 0 0 1 3.4 4.4A3.5 3.5 0 0 1 20 10a3.5 3.5 0 0 1-1.8 3.1A3.5 3.5 0 0 1 17 17a3.5 3.5 0 0 1-3.5 3.5c-.8 0-1.5-.2-2.1-.6" />
              <path d="M12 2v20" />
            </svg>
            <span className="main-title">
              {geminiSource === "screenshot" ? "Analyzing screenshot..." : "Thinking..."}
            </span>
          </div>
        </div>
        <div className="main-body main-processing-body">
          <div className="brain-loading">
            <svg
              className="brain-loading-icon"
              width="28"
              height="28"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M9.5 2a3.5 3.5 0 0 0-3.4 4.4A3.5 3.5 0 0 0 4 10a3.5 3.5 0 0 0 1.8 3.1A3.5 3.5 0 0 0 7 17a3.5 3.5 0 0 0 3.5 3.5c.8 0 1.5-.2 2.1-.6" />
              <path d="M14.5 2a3.5 3.5 0 0 1 3.4 4.4A3.5 3.5 0 0 1 20 10a3.5 3.5 0 0 1-1.8 3.1A3.5 3.5 0 0 1 17 17a3.5 3.5 0 0 1-3.5 3.5c-.8 0-1.5-.2-2.1-.6" />
              <path d="M12 2v20" />
            </svg>
          </div>
        </div>
      </div>
    );
  }

  // --- RESPONSE ---
  const responseLabel = geminiSource === "screenshot" ? "Screenshot" : "Response";

  return (
    <div className="main-panel" ref={contentRef}>
      <div className="main-titlebar" data-tauri-drag-region>
        <div className="main-title-left">
          {geminiSource === "screenshot" ? (
            <svg className="main-title-icon screenshot-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
              <circle cx="8.5" cy="8.5" r="1.5" />
              <polyline points="21 15 16 10 5 21" />
            </svg>
          ) : (
            <svg className="main-title-icon sparkle-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
            </svg>
          )}
          <span className="main-title">{responseLabel}</span>
          {model && <span className="model-badge">{model.replace("gemini-", "")}</span>}
        </div>
        <div className="main-title-right">
          {historyCount > 1 && (
            <div className="history-nav">
              <button
                className="history-nav-btn"
                onClick={goBack}
                disabled={!canGoBack}
              >
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                  <polyline points="15 18 9 12 15 6" />
                </svg>
              </button>
              <span className="history-counter">{historyIndex + 1}/{historyCount}</span>
              <button
                className="history-nav-btn"
                onClick={goForward}
                disabled={!canGoForward}
              >
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                  <polyline points="9 18 15 12 9 6" />
                </svg>
              </button>
            </div>
          )}
          <button className="main-copy-btn" onClick={() => {
            if (response) {
              navigator.clipboard.writeText(response);
              setCopied(true);
              setTimeout(() => setCopied(false), 1500);
            }
          }}>
            {copied ? (
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <polyline points="20 6 9 17 4 12" />
              </svg>
            ) : (
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
              </svg>
            )}
          </button>
          <button className="main-close-btn" onClick={handleClear}>
            <svg width="10" height="10" viewBox="0 0 10 10">
              <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
          </button>
        </div>
      </div>
      {transcript && (
        <>
          <div className="main-body" ref={scrollRef}>
            <div className="main-transcript">{transcript}</div>
          </div>
          <div className="main-response-divider" />
        </>
      )}
      <div className={transcript ? "main-response-area" : "main-body"} ref={responseRef}>
        <div className="markdown-body">
          <Markdown>{response}</Markdown>
        </div>
      </div>
    </div>
  );
}

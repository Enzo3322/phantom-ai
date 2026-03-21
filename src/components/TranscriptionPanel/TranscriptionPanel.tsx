import { useRef, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Markdown from "react-markdown";
import { GlassContainer } from "../shared/GlassContainer";
import { LoadingSpinner } from "../shared/LoadingSpinner";
import { useTranscription } from "../../hooks/useTranscription";
import { useGemini } from "../../hooks/useGemini";
import "./TranscriptionPanel.css";

export function TranscriptionPanel() {
  const { transcript, isRecording, error, clearTranscript } = useTranscription();
  const { response, loading: geminiLoading } = useGemini();
  const scrollRef = useRef<HTMLDivElement>(null);
  const [sending, setSending] = useState(false);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [transcript]);

  const handleSendToGemini = async () => {
    if (!transcript.trim()) return;
    setSending(true);
    try {
      await invoke("send_transcription_to_gemini", {
        text: transcript,
        prompt: "Analyze the following audio transcription and provide a helpful response.",
      });
    } catch (e) {
      console.error("Send to Gemini failed:", e);
    } finally {
      setSending(false);
    }
  };

  const handleToggleRecording = async () => {
    try {
      if (isRecording) {
        await invoke("stop_recording");
      } else {
        await invoke("start_recording");
      }
    } catch (e) {
      console.error("Toggle recording failed:", e);
    }
  };

  const handleClear = async () => {
    if (isRecording) {
      await invoke("stop_recording");
    }
    clearTranscript();
  };

  return (
    <GlassContainer>
      <div className="transcription-titlebar" data-tauri-drag-region>
        <div className="transcription-title-left">
          <span
            className={`rec-indicator ${isRecording ? "active" : ""}`}
          />
          <span className="transcription-title">
            {isRecording ? "Recording..." : "Transcription"}
          </span>
        </div>
        <div className="transcription-title-right">
          <button
            className={`rec-toggle-btn ${isRecording ? "recording" : ""}`}
            onClick={handleToggleRecording}
          >
            {isRecording ? "Stop" : "Start"}
          </button>
        </div>
      </div>

      <div className="transcription-body" ref={scrollRef}>
        {error && (
          <div className="transcription-error">
            <p>{error}</p>
          </div>
        )}

        {transcript ? (
          <div className="transcription-text">{transcript}</div>
        ) : (
          !isRecording &&
          !error && (
            <div className="transcription-empty">
              <p>
                Press <kbd>⌘ ⇧ M</kbd> to start recording
              </p>
            </div>
          )
        )}

        {isRecording && !transcript && (
          <div className="transcription-listening">
            <LoadingSpinner />
            <span>Listening...</span>
          </div>
        )}
      </div>

      {transcript && !isRecording && (
        <div className="transcription-actions">
          <button
            className="action-btn primary"
            onClick={handleSendToGemini}
            disabled={sending || geminiLoading}
          >
            {sending || geminiLoading ? "Sending..." : "Send to Gemini"}
          </button>
          <button className="action-btn secondary" onClick={handleClear}>
            Clear
          </button>
        </div>
      )}

      {(geminiLoading || response) && (
        <div className="transcription-gemini">
          <div className="gemini-divider" />
          {geminiLoading && <LoadingSpinner />}
          {response && !geminiLoading && (
            <div className="markdown-body">
              <Markdown>{response}</Markdown>
            </div>
          )}
        </div>
      )}
    </GlassContainer>
  );
}

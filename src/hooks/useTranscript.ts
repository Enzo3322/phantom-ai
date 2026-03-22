import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export function useTranscript() {
  const [transcript, setTranscript] = useState("");
  const [preview, setPreview] = useState("");
  const [isComplete, setIsComplete] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<string>("get_transcription").then((text) => {
      if (text) setTranscript(text);
    });
  }, []);

  useEffect(() => {
    const listeners = [
      listen("recording-started", () => {
        setTranscript("");
        setPreview("");
        setIsComplete(false);
        setError(null);
      }),
      listen<string>("transcription-partial", (event) => {
        setTranscript(event.payload);
        setPreview("");
        setError(null);
      }),
      listen<string>("transcription-preview", (event) => {
        setPreview(event.payload);
      }),
      listen<string>("transcription-complete", (event) => {
        setTranscript(event.payload);
        setPreview("");
        setIsComplete(true);
      }),
      listen<string>("transcription-error", (event) => {
        setError(event.payload);
      }),
    ];

    return () => {
      listeners.forEach((p) => p.then((unlisten) => unlisten()));
    };
  }, []);

  const clearTranscript = useCallback(() => {
    setTranscript("");
    setPreview("");
    setError(null);
  }, []);

  return { transcript, preview, isComplete, error, clearTranscript };
}

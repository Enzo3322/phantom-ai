import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export function useTranscript() {
  const [transcript, setTranscript] = useState("");
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
        setIsComplete(false);
        setError(null);
      }),
      listen<string>("transcription-partial", (event) => {
        setTranscript(event.payload);
        setError(null);
      }),
      listen<string>("transcription-complete", (event) => {
        setTranscript(event.payload);
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
    setError(null);
  }, []);

  return { transcript, isComplete, error, clearTranscript };
}

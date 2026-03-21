import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export function useTranscription() {
  const [transcript, setTranscript] = useState<string>("");
  const [isRecording, setIsRecording] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<boolean>("get_recording_status").then(setIsRecording);
    invoke<string>("get_transcription").then((text) => {
      if (text) setTranscript(text);
    });
  }, []);

  useEffect(() => {
    const listeners = [
      listen<undefined>("recording-started", () => {
        setIsRecording(true);
        setError(null);
        setTranscript("");
      }),
      listen<undefined>("recording-stopped", () => {
        setIsRecording(false);
      }),
      listen<string>("transcription-partial", (event) => {
        setTranscript(event.payload);
        setError(null);
      }),
      listen<string>("transcription-complete", (event) => {
        setTranscript(event.payload);
        setIsRecording(false);
      }),
      listen<string>("transcription-error", (event) => {
        setError(event.payload);
        setIsRecording(false);
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

  return { transcript, isRecording, error, clearTranscript };
}

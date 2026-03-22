import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type GeminiSource = "screenshot" | "transcription" | "watcher" | null;

interface CaptureResponsePayload {
  text: string;
  source: GeminiSource;
  model?: string;
}

export function useGemini() {
  const [response, setResponse] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [source, setSource] = useState<GeminiSource>(null);
  const [model, setModel] = useState<string | null>(null);

  useEffect(() => {
    invoke<boolean>("get_processing_status").then((processing) => {
      if (processing) setLoading(true);
    });
    invoke<string | null>("get_last_response").then((last) => {
      if (last) {
        if (last.startsWith("Error:")) {
          setError(last);
        } else {
          setResponse(last);
        }
      }
    });
  }, []);

  useEffect(() => {
    const listeners = [
      listen<string>("processing-start", (event) => {
        setLoading(true);
        setError(null);
        setResponse(null);
        const src = event.payload;
        setSource(src === "screenshot" ? "screenshot" : src === "watcher" ? "watcher" : "transcription");
      }),
      listen<CaptureResponsePayload>("capture-response", (event) => {
        setLoading(false);
        setResponse(event.payload.text);
        setSource(event.payload.source);
        setModel(event.payload.model ?? null);
        setError(null);
      }),
      listen<string>("capture-error", (event) => {
        setLoading(false);
        setError(event.payload);
      }),
    ];

    return () => {
      listeners.forEach((p) => p.then((unlisten) => unlisten()));
    };
  }, []);

  const clearResponse = useCallback(() => {
    setResponse(null);
    setLoading(false);
    setError(null);
    setSource(null);
    setModel(null);
  }, []);

  return { response, loading, error, source, model, clearResponse };
}

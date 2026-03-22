import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type GeminiSource = "screenshot" | "transcription" | null;

interface CaptureResponsePayload {
  text: string;
  source: GeminiSource;
  model?: string;
}

interface HistoryEntry {
  text: string;
  source: GeminiSource;
  model: string | null;
}

export function useGemini() {
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [currentIndex, setCurrentIndex] = useState(-1);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<boolean>("get_processing_status").then((processing) => {
      if (processing) setLoading(true);
    });
    invoke<Array<{ id: number; timestamp: string; source: string; model: string; response: string }>>(
      "get_response_history"
    ).then((entries) => {
      if (entries.length > 0) {
        const loaded = entries.map((e) => ({
          text: e.response,
          source: (e.source === "screenshot" ? "screenshot" : "transcription") as GeminiSource,
          model: e.model,
        }));
        setHistory(loaded);
        setCurrentIndex(loaded.length - 1);
      }
    });
  }, []);

  useEffect(() => {
    const listeners = [
      listen<string>("processing-start", () => {
        setLoading(true);
        setError(null);
      }),
      listen<CaptureResponsePayload>("capture-response", (event) => {
        setLoading(false);
        setError(null);
        const entry: HistoryEntry = {
          text: event.payload.text,
          source: event.payload.source,
          model: event.payload.model ?? null,
        };
        setHistory((prev) => {
          const next = [...prev, entry];
          setCurrentIndex(next.length - 1);
          return next;
        });
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

  const goBack = useCallback(() => {
    setCurrentIndex((i) => Math.max(0, i - 1));
  }, []);

  const goForward = useCallback(() => {
    setCurrentIndex((i) => Math.min(history.length - 1, i + 1));
  }, [history.length]);

  const current = currentIndex >= 0 && currentIndex < history.length ? history[currentIndex] : null;

  const clearResponse = useCallback(() => {
    setCurrentIndex(-1);
    setLoading(false);
    setError(null);
  }, []);

  return {
    response: current?.text ?? null,
    source: current?.source ?? null,
    model: current?.model ?? null,
    loading,
    error,
    clearResponse,
    goBack,
    goForward,
    canGoBack: currentIndex > 0,
    canGoForward: currentIndex < history.length - 1,
    historyCount: history.length,
    historyIndex: currentIndex,
  };
}

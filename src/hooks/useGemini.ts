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
    let cancelled = false;
    const unlisteners: Array<() => void> = [];

    async function setup() {
      const u1 = await listen<string>("processing-start", () => {
        if (!cancelled) {
          setLoading(true);
          setError(null);
        }
      });
      if (cancelled) { u1(); return; }
      unlisteners.push(u1);

      const u2 = await listen<CaptureResponsePayload>("capture-response", (event) => {
        if (!cancelled) {
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
        }
      });
      if (cancelled) { u2(); return; }
      unlisteners.push(u2);

      const u3 = await listen<string>("capture-error", (event) => {
        if (!cancelled) {
          setLoading(false);
          setError(event.payload);
        }
      });
      if (cancelled) { u3(); return; }
      unlisteners.push(u3);
    }

    setup();

    return () => {
      cancelled = true;
      unlisteners.forEach((fn) => fn());
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

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export function useGemini() {
  const [response, setResponse] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Fetch current state on mount (handles case where events were missed)
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

  // Listen for real-time updates
  useEffect(() => {
    const listeners = [
      listen<undefined>("processing-start", () => {
        setLoading(true);
        setError(null);
        setResponse(null);
      }),
      listen<string>("capture-response", (event) => {
        setLoading(false);
        setResponse(event.payload);
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

  return { response, loading, error };
}

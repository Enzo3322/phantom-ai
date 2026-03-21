import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

export function useGemini() {
  const [response, setResponse] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

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

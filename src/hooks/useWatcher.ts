import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface WatcherTickPayload {
  status: "no_change" | "processing";
}

export function useWatcher() {
  const [active, setActive] = useState(false);
  const [lastTick, setLastTick] = useState<string | null>(null);

  useEffect(() => {
    invoke<boolean>("get_watcher_status").then(setActive);
  }, []);

  useEffect(() => {
    const listeners = [
      listen("watcher-started", () => {
        setActive(true);
        setLastTick(null);
      }),
      listen("watcher-stopped", () => {
        setActive(false);
        setLastTick(null);
      }),
      listen<WatcherTickPayload>("watcher-ocr-tick", (event) => {
        setLastTick(event.payload.status);
      }),
    ];

    return () => {
      listeners.forEach((p) => p.then((unlisten) => unlisten()));
    };
  }, []);

  const toggleWatcher = useCallback(async () => {
    await invoke("toggle_watcher");
  }, []);

  return { active, lastTick, toggleWatcher };
}

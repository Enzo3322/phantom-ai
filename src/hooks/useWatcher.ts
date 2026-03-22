import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type WatcherStage =
  | "idle"
  | "capturing"
  | "extracting"
  | "analyzing"
  | "generating";

const STAGE_LABELS: Record<WatcherStage, string> = {
  idle: "Watching...",
  capturing: "Capturing screen...",
  extracting: "Extracting content...",
  analyzing: "Analyzing content...",
  generating: "Generating response...",
};

export function useWatcher() {
  const [active, setActive] = useState(false);
  const [stage, setStage] = useState<WatcherStage>("idle");

  useEffect(() => {
    invoke<boolean>("get_watcher_status").then(setActive);
  }, []);

  useEffect(() => {
    const listeners = [
      listen("watcher-started", () => {
        setActive(true);
        setStage("idle");
      }),
      listen("watcher-stopped", () => {
        setActive(false);
        setStage("idle");
      }),
      listen<string>("watcher-stage", (event) => {
        const s = event.payload;
        if (s in STAGE_LABELS) {
          setStage(s as WatcherStage);
        }
      }),
    ];

    return () => {
      listeners.forEach((p) => p.then((unlisten) => unlisten()));
    };
  }, []);

  const toggleWatcher = useCallback(async () => {
    await invoke("toggle_watcher");
  }, []);

  const stageLabel = STAGE_LABELS[stage];

  return { active, stage, stageLabel, toggleWatcher };
}

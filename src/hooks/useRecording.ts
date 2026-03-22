import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export function useRecording() {
  const [isRecording, setIsRecording] = useState(false);

  useEffect(() => {
    invoke<boolean>("get_recording_status").then(setIsRecording);
  }, []);

  useEffect(() => {
    const listeners = [
      listen("recording-started", () => setIsRecording(true)),
      listen("recording-stopped", () => setIsRecording(false)),
    ];

    return () => {
      listeners.forEach((p) => p.then((unlisten) => unlisten()));
    };
  }, []);

  const startRecording = useCallback(() => invoke("start_recording"), []);
  const stopRecording = useCallback(() => invoke("stop_recording"), []);

  const toggleRecording = useCallback(async () => {
    const recording = await invoke<boolean>("get_recording_status");
    if (recording) await invoke("stop_recording");
    else await invoke("start_recording");
  }, []);

  return { isRecording, startRecording, stopRecording, toggleRecording };
}

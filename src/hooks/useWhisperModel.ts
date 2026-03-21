import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface ModelInfo {
  size: string;
  label: string;
  downloaded: boolean;
  file_size_mb: number;
}

interface DownloadProgress {
  size: string;
  progress: number;
}

export function useWhisperModel() {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [downloading, setDownloading] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState(0);

  const refreshModels = useCallback(() => {
    invoke<ModelInfo[]>("get_available_models").then(setModels);
  }, []);

  useEffect(() => {
    refreshModels();
  }, [refreshModels]);

  useEffect(() => {
    const listeners = [
      listen<DownloadProgress>("model-download-progress", (event) => {
        setDownloading(event.payload.size);
        setDownloadProgress(event.payload.progress);
      }),
      listen<string>("model-download-complete", () => {
        setDownloading(null);
        setDownloadProgress(0);
        refreshModels();
      }),
    ];

    return () => {
      listeners.forEach((p) => p.then((unlisten) => unlisten()));
    };
  }, [refreshModels]);

  const downloadModel = useCallback(async (size: string) => {
    setDownloading(size);
    setDownloadProgress(0);
    try {
      await invoke("download_whisper_model", { size });
      refreshModels();
    } catch (e) {
      console.error("Download failed:", e);
    } finally {
      setDownloading(null);
      setDownloadProgress(0);
    }
  }, [refreshModels]);

  return { models, downloading, downloadProgress, downloadModel };
}

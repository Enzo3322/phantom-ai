import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { ConfigPanel } from "./components/ConfigPanel/ConfigPanel";
import { ResponsePanel } from "./components/ResponsePanel/ResponsePanel";
import { TranscriptionPanel } from "./components/TranscriptionPanel/TranscriptionPanel";

function App() {
  const label = getCurrentWindow().label;

  useEffect(() => {
    invoke<{ opacity: number }>("get_config").then(({ opacity }) => {
      document.documentElement.style.setProperty("--bg-opacity", String(opacity));
    });
  }, []);

  if (label === "config") return <ConfigPanel />;
  if (label === "response") return <ResponsePanel />;
  if (label === "transcription") return <TranscriptionPanel />;

  return null;
}

export default App;

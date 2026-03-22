import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { ConfigPanel } from "./components/ConfigPanel/ConfigPanel";
import { MainPanel } from "./components/MainPanel/MainPanel";
import { WelcomePanel } from "./components/WelcomePanel/WelcomePanel";

function App() {
  const label = getCurrentWindow().label;

  useEffect(() => {
    invoke<{ opacity: number }>("get_config").then(({ opacity }) => {
      document.documentElement.style.setProperty("--bg-opacity", String(opacity));
    });
  }, []);

  if (label === "config") return <ConfigPanel />;
  if (label === "main") return <MainPanel />;
  if (label === "welcome") return <WelcomePanel />;

  return null;
}

export default App;

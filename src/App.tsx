import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { ConfigPanel } from "./components/ConfigPanel/ConfigPanel";
import { ResponsePanel } from "./components/ResponsePanel/ResponsePanel";

function App() {
  const label = getCurrentWindow().label;

  useEffect(() => {
    invoke<{ glass_effect: boolean }>("get_config").then(({ glass_effect }) => {
      if (!glass_effect) {
        document.documentElement.setAttribute("data-theme", "solid");
      }
    });
  }, []);

  if (label === "config") return <ConfigPanel />;
  if (label === "response") return <ResponsePanel />;

  return null;
}

export default App;

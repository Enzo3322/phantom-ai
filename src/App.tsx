import { getCurrentWindow } from "@tauri-apps/api/window";
import { ConfigPanel } from "./components/ConfigPanel/ConfigPanel";
import { ResponsePanel } from "./components/ResponsePanel/ResponsePanel";

function App() {
  const label = getCurrentWindow().label;

  if (label === "config") return <ConfigPanel />;
  if (label === "response") return <ResponsePanel />;

  return null;
}

export default App;

import { getCurrentWindow } from "@tauri-apps/api/window";
import { ConfigPanel } from "./components/ConfigPanel/ConfigPanel";
import { MainPanel } from "./components/MainPanel/MainPanel";
import { WelcomePanel } from "./components/WelcomePanel/WelcomePanel";

function App() {
  const label = getCurrentWindow().label;

  if (label === "config") return <ConfigPanel />;
  if (label === "main") return <MainPanel />;
  if (label === "welcome") return <WelcomePanel />;

  return null;
}

export default App;

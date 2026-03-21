import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { load } from "@tauri-apps/plugin-store";

interface Config {
  api_key: string;
  model: string;
  prompt: string;
  glass_effect: boolean;
}

export function useConfig() {
  const [config, setConfig] = useState<Config>({
    api_key: "",
    model: "gemini-2.0-flash",
    prompt:
      "Analyze this screenshot and answer any questions visible on screen. Be concise and direct.",
    glass_effect: true,
  });
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<Config>("get_config").then(setConfig);
  }, []);

  const save = useCallback(
    async (newConfig: Config) => {
      setSaving(true);
      try {
        await invoke("save_config", {
          apiKey: newConfig.api_key,
          model: newConfig.model,
          prompt: newConfig.prompt,
          glassEffect: newConfig.glass_effect,
        });

        const store = await load("config.json");
        await store.set("api_key", newConfig.api_key);
        await store.set("model", newConfig.model);
        await store.set("prompt", newConfig.prompt);
        await store.set("glass_effect", newConfig.glass_effect);
        await store.save();

        const glassChanged = newConfig.glass_effect !== config.glass_effect;
        setConfig(newConfig);
        setSaved(true);
        setTimeout(() => setSaved(false), 2000);

        if (glassChanged) {
          // Update current window theme immediately
          if (newConfig.glass_effect) {
            document.documentElement.removeAttribute("data-theme");
          } else {
            document.documentElement.setAttribute("data-theme", "solid");
          }
          // Rebuild other windows (not the current one)
          await invoke("rebuild_windows", { caller: getCurrentWindow().label });
        }
      } finally {
        setSaving(false);
      }
    },
    [config.glass_effect]
  );

  return { config, save, saving, saved };
}

import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { load } from "@tauri-apps/plugin-store";

interface Config {
  api_key: string;
  model: string;
  prompt: string;
  opacity: number;
  stealth_mode: boolean;
  whisper_model_size: string;
  whisper_language: string;
  audio_source: string;
  vocab_seed: string;
}

export function useConfig() {
  const [config, setConfig] = useState<Config>({
    api_key: "",
    model: "gemini-2.0-flash",
    prompt:
      "Analyze this screenshot and answer any questions visible on screen. Be concise and direct.",
    opacity: 0.85,
    stealth_mode: true,
    whisper_model_size: "small",
    whisper_language: "auto",
    audio_source: "both",
    vocab_seed: "",
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
          opacity: newConfig.opacity,
          stealthMode: newConfig.stealth_mode,
          whisperModelSize: newConfig.whisper_model_size,
          whisperLanguage: newConfig.whisper_language,
          audioSource: newConfig.audio_source,
          vocabSeed: newConfig.vocab_seed,
        });

        const store = await load("config.json");
        await store.set("api_key", newConfig.api_key);
        await store.set("model", newConfig.model);
        await store.set("prompt", newConfig.prompt);
        await store.set("opacity", newConfig.opacity);
        await store.set("stealth_mode", newConfig.stealth_mode);
        await store.set("whisper_model_size", newConfig.whisper_model_size);
        await store.set("whisper_language", newConfig.whisper_language);
        await store.set("audio_source", newConfig.audio_source);
        await store.set("vocab_seed", newConfig.vocab_seed);
        await store.save();

        setConfig(newConfig);
        setSaved(true);
        setTimeout(() => setSaved(false), 2000);
      } finally {
        setSaving(false);
      }
    },
    []
  );

  return { config, save, saving, saved };
}

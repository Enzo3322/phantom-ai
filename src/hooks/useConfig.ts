import { useState, useEffect, useCallback, useRef } from "react";
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
  response_language: string;
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
    response_language: "auto",
  });
  const [autoSaved, setAutoSaved] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => {
    invoke<Config>("get_config").then(setConfig);
  }, []);

  const persistConfig = useCallback(async (newConfig: Config) => {
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
      responseLanguage: newConfig.response_language,
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
    await store.set("response_language", newConfig.response_language);
    await store.save();

    setAutoSaved(true);
    setTimeout(() => setAutoSaved(false), 1500);
  }, []);

  const updateConfig = useCallback(
    (partial: Partial<Config>) => {
      setConfig((prev) => {
        const next = { ...prev, ...partial };

        if (debounceRef.current) clearTimeout(debounceRef.current);
        debounceRef.current = setTimeout(() => {
          persistConfig(next);
        }, 500);

        return next;
      });
    },
    [persistConfig]
  );

  useEffect(() => {
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, []);

  return { config, updateConfig, autoSaved };
}

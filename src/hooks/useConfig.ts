import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { load } from "@tauri-apps/plugin-store";

interface Config {
  api_key: string;
  model: string;
  prompt: string;
  stealth_mode: boolean;
  whisper_model_size: string;
  whisper_language: string;
  audio_source: string;
  vocab_seed: string;
  response_language: string;
  dodge_on_hover: boolean;
  process_disguise_name: string;
  passthrough_mode: boolean;
  network_jitter: boolean;
  proxy_url: string;
  spoof_user_agent: boolean;
}

export function useConfig() {
  const [config, setConfig] = useState<Config>({
    api_key: "",
    model: "gemini-2.5-flash-lite",
    prompt:
      "Analyze this screenshot and answer any questions visible on screen. Be concise and direct.",
    stealth_mode: true,
    whisper_model_size: "small",
    whisper_language: "auto",
    audio_source: "both",
    vocab_seed: "",
    response_language: "auto",
    dodge_on_hover: false,
    process_disguise_name: "",
    passthrough_mode: false,
    network_jitter: true,
    proxy_url: "",
    spoof_user_agent: true,
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
      stealthMode: newConfig.stealth_mode,
      whisperModelSize: newConfig.whisper_model_size,
      whisperLanguage: newConfig.whisper_language,
      audioSource: newConfig.audio_source,
      vocabSeed: newConfig.vocab_seed,
      responseLanguage: newConfig.response_language,
      dodgeOnHover: newConfig.dodge_on_hover,
      processDisguiseName: newConfig.process_disguise_name,
      passthroughMode: newConfig.passthrough_mode,
      networkJitter: newConfig.network_jitter,
      proxyUrl: newConfig.proxy_url,
      spoofUserAgent: newConfig.spoof_user_agent,
    });

    const store = await load("config.json");
    await store.set("api_key", newConfig.api_key);
    await store.set("model", newConfig.model);
    await store.set("prompt", newConfig.prompt);
    await store.set("stealth_mode", newConfig.stealth_mode);
    await store.set("whisper_model_size", newConfig.whisper_model_size);
    await store.set("whisper_language", newConfig.whisper_language);
    await store.set("audio_source", newConfig.audio_source);
    await store.set("vocab_seed", newConfig.vocab_seed);
    await store.set("response_language", newConfig.response_language);
    await store.set("dodge_on_hover", newConfig.dodge_on_hover);
    await store.set("process_disguise_name", newConfig.process_disguise_name);
    await store.set("passthrough_mode", newConfig.passthrough_mode);
    await store.set("network_jitter", newConfig.network_jitter);
    await store.set("proxy_url", newConfig.proxy_url);
    await store.set("spoof_user_agent", newConfig.spoof_user_agent);
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

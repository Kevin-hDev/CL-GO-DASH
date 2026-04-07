import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

export function useOllamaStatus() {
  const [running, setRunning] = useState(false);

  useEffect(() => {
    invoke<boolean>("is_ollama_running").then(setRunning).catch(() => {});

    const unlisten = listen<boolean>("ollama-status", (e) => {
      setRunning(e.payload);
    });

    return () => { unlisten.then((fn) => fn()).catch(() => {}); };
  }, []);

  return running;
}

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useSettingValue<T>(key: string, fallback: T): T {
  const [value, setValue] = useState<T>(fallback);

  useEffect(() => {
    invoke<Record<string, unknown>>("get_advanced_settings")
      .then((s) => {
        if (key in s) setValue(s[key] as T);
      })
      .catch(() => {});
  }, [key]);

  return value;
}

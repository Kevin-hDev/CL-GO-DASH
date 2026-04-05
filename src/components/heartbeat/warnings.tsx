import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";

interface LogEntry {
  timestamp: string;
  message: string;
  is_error: boolean;
}

export function Warnings() {
  const { t } = useTranslation();
  const [entries, setEntries] = useState<LogEntry[]>([]);

  useEffect(() => {
    async function load() {
      try {
        const data = await invoke<LogEntry[]>("get_warnings");
        setEntries(data);
      } catch (e) {
        // Warnings may not be available — silent fail
      }
    }
    load();
  }, []);

  if (entries.length < 1) {
    return (
      <div className="p-6 text-sm text-[var(--ink-faint)]">
        {t("heartbeat.noErrors")}
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-2 p-6">
      {entries.map((entry, i) => (
        <div
          key={i}
          className="flex gap-2 px-4 py-2 border-l-2 border-[var(--signal-error)] bg-[rgba(217,68,68,0.05)] rounded-r"
        >
          <span className="font-mono text-xs text-[var(--ink-faint)] whitespace-nowrap shrink-0">
            {entry.timestamp}
          </span>
          <span className="text-sm text-[var(--signal-error)]">
            {entry.message}
          </span>
        </div>
      ))}
    </div>
  );
}

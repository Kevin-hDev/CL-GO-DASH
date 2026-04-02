import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./warnings.css";

interface LogEntry {
  timestamp: string;
  message: string;
  is_error: boolean;
}

export function Warnings() {
  const [entries, setEntries] = useState<LogEntry[]>([]);

  useEffect(() => {
    async function load() {
      try {
        const data = await invoke<LogEntry[]>("get_warnings");
        setEntries(data);
      } catch (e) {
        console.error("Failed to load warnings:", e);
      }
    }
    load();
  }, []);

  if (entries.length < 1) {
    return (
      <div className="warnings-empty">
        Aucune erreur détectée
      </div>
    );
  }

  return (
    <div className="warnings-content">
      {entries.map((entry, i) => (
        <div key={i} className="warning-entry">
          <span className="warning-time">{entry.timestamp}</span>
          <span className="warning-msg">{entry.message}</span>
        </div>
      ))}
    </div>
  );
}

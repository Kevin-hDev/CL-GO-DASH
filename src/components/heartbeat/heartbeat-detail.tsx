import { useState, useEffect } from "react";
import type { ScheduledWakeup } from "@/types/config";
import { ModeSelector } from "./mode-selector";
import { SignalDot } from "./signal-dot";
import "./heartbeat-detail.css";

interface HeartbeatDetailProps {
  wakeup: ScheduledWakeup;
  onSave: (wakeup: ScheduledWakeup) => void;
  onDelete: (id: string) => void;
  onRun: (id: string) => void;
}

export function HeartbeatDetail({
  wakeup,
  onSave,
  onDelete,
  onRun,
}: HeartbeatDetailProps) {
  const [time, setTime] = useState(wakeup.time);
  const [mode, setMode] = useState(wakeup.mode);
  const [prompt, setPrompt] = useState(wakeup.prompt ?? "");
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    setTime(wakeup.time);
    setMode(wakeup.mode);
    setPrompt(wakeup.prompt ?? "");
    setSaved(false);
  }, [wakeup]);

  function handleSave() {
    onSave({ ...wakeup, time, mode, prompt: prompt || null });
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  }

  return (
    <>
      <div className="detail-header">
        <div className="detail-title-row">
          <SignalDot state={wakeup.active ? "ok" : "idle"} />
          <div className="detail-title">
            Réveil {wakeup.time}
            <span className="detail-status">
              {wakeup.active ? " · actif" : " · inactif"}
            </span>
          </div>
        </div>
        <div className="detail-actions">
          <button className="btn btn-primary" onClick={() => onRun(wakeup.id)}>
            ▶ Run
          </button>
          <button className="btn" onClick={handleSave}>
            {saved ? "✓ Sauvegardé" : "Sauvegarder"}
          </button>
          <button className="btn btn-danger" onClick={() => onDelete(wakeup.id)}>
            Supprimer
          </button>
        </div>
      </div>
      <div className="detail-content">
        <div className="form-card">
          <div className="form-group">
            <label className="form-label">Heure</label>
            <input
              type="time"
              className="form-input"
              value={time}
              onChange={(e) => setTime(e.target.value)}
            />
          </div>
          <div className="form-group">
            <label className="form-label">Mode</label>
            <ModeSelector value={mode} onChange={setMode} />
          </div>
          <div className="form-group">
            <label className="form-label">Prompt (optionnel)</label>
            <textarea
              className="prompt-area"
              placeholder="Chargé en contexte au réveil..."
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
            />
          </div>
        </div>
      </div>
    </>
  );
}

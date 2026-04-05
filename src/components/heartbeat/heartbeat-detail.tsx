import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import type { ScheduledWakeup } from "@/types/config";
import { ModeSelector } from "./mode-selector";
import { SignalDot } from "./signal-dot";
import { DatetimeInput } from "@/components/ui/datetime-input";
import { Play, Check, Trash } from "@/components/ui/icons";
import { cn } from "@/lib/utils";

interface HeartbeatDetailProps {
  wakeup: ScheduledWakeup;
  onSave: (wakeup: ScheduledWakeup) => void;
  onToggleActive: (wakeup: ScheduledWakeup) => void;
  onDelete: (id: string) => void;
  onRun: (id: string) => void;
}

export function HeartbeatDetail({
  wakeup, onSave, onToggleActive, onDelete, onRun,
}: HeartbeatDetailProps) {
  const { t } = useTranslation();
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
      {/* Header */}
      <div
        className="flex items-center justify-between"
        style={{
          padding: "16px 24px",
          borderBottom: "1px solid var(--edge)",
        }}
      >
        <div className="flex items-center gap-2">
          <SignalDot state={wakeup.active ? "ok" : "idle"} />
          <div
            className={`toggle ${wakeup.active ? "on" : ""}`}
            onClick={() => onToggleActive({ ...wakeup, active: !wakeup.active })}
          />
          <span style={{ fontSize: "var(--text-lg)", fontWeight: 600 }}>
            {wakeup.active ? t("heartbeat.active") : t("heartbeat.inactive")}
          </span>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => onRun(wakeup.id)}
            className={cn(
              "flex items-center gap-1 cursor-pointer",
              "hover:opacity-90 transition-opacity duration-200",
            )}
            style={{
              padding: "6px 12px",
              fontSize: "var(--text-sm)",
              borderRadius: "var(--radius-sm)",
              background: "var(--pulse)",
              color: "#fff",
              border: "1px solid var(--pulse)",
            }}
          >
            <Play size={14} weight="fill" /> {t("heartbeat.run")}
          </button>
          <button
            onClick={handleSave}
            className="flex items-center gap-1 cursor-pointer hover:opacity-80 transition-opacity duration-200"
            style={{
              padding: "6px 12px",
              fontSize: "var(--text-sm)",
              borderRadius: "var(--radius-sm)",
              background: "var(--surface)",
              color: "var(--ink)",
              border: "1px solid var(--edge)",
            }}
          >
            {saved ? <><Check size={14} /> {t("heartbeat.saved")}</> : t("heartbeat.save")}
          </button>
          <button
            onClick={() => onDelete(wakeup.id)}
            className="flex items-center gap-1 cursor-pointer hover:opacity-80 transition-opacity duration-200"
            style={{
              padding: "6px 12px",
              fontSize: "var(--text-sm)",
              borderRadius: "var(--radius-sm)",
              color: "var(--signal-error)",
              border: "1px solid rgba(217, 68, 68, 0.3)",
              background: "transparent",
            }}
          >
            <Trash size={14} /> {t("heartbeat.delete")}
          </button>
        </div>
      </div>

      {/* Form */}
      <div className="flex-1 overflow-y-auto" style={{ padding: 24 }}>
        <div style={{
          background: "var(--surface)",
          border: "1px solid var(--edge)",
          borderRadius: "var(--radius-md)",
          padding: 24,
          boxShadow: "var(--shadow-card)",
          maxWidth: 520,
        }}>
          {/* Time */}
          <div style={{ marginBottom: 20 }}>
            <label style={{
              display: "block",
              fontSize: "var(--text-xs)",
              color: "var(--ink-muted)",
              textTransform: "uppercase",
              letterSpacing: "0.5px",
              marginBottom: 8,
            }}>
              {t("heartbeat.dateTime")}
            </label>
            <DatetimeInput
              value={time}
              onChange={setTime}
              className="form-input"
            />
          </div>

          {/* Mode */}
          <div style={{ marginBottom: 20 }}>
            <label style={{
              display: "block",
              fontSize: "var(--text-xs)",
              color: "var(--ink-muted)",
              textTransform: "uppercase",
              letterSpacing: "0.5px",
              marginBottom: 8,
            }}>
              {t("heartbeat.mode")}
            </label>
            <ModeSelector value={mode} onChange={setMode} />
          </div>

          {/* Prompt */}
          <div>
            <label style={{
              display: "block",
              fontSize: "var(--text-xs)",
              color: "var(--ink-muted)",
              textTransform: "uppercase",
              letterSpacing: "0.5px",
              marginBottom: 8,
            }}>
              {t("heartbeat.promptLabel")}
            </label>
            <textarea
              placeholder={t("heartbeat.promptPlaceholder")}
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              style={{
                width: "100%",
                minHeight: 80,
                padding: 8,
                fontSize: "var(--text-sm)",
                fontFamily: "var(--font-mono)",
                background: "var(--void)",
                border: "1px solid var(--edge)",
                borderRadius: "var(--radius-sm)",
                color: "var(--ink)",
                resize: "vertical",
              }}
            />
          </div>
        </div>
      </div>
    </>
  );
}

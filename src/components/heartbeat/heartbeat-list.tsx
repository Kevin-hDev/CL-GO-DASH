import { useCallback, useRef } from "react";
import type { ScheduledWakeup } from "@/types/config";
import { SignalDot } from "./signal-dot";
import { DatetimeInput } from "@/components/ui/datetime-input";
import "./heartbeat-list.css";

interface HeartbeatListProps {
  wakeups: ScheduledWakeup[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onAdd: () => void;
  heartbeatActive: boolean;
  onToggleHeartbeat: (active: boolean) => void;
  stopAt: string | null;
  onStopAtChange: (value: string | null) => void;
  sessionRunning: boolean;
  onContextMenu: (e: React.MouseEvent, id: string) => void;
  activeSubTab: "planned" | "warning";
  onSubTabChange: (tab: "planned" | "warning") => void;
}

function formatWakeupTime(time: string): string {
  if (time.includes("T")) {
    const d = new Date(time);
    const months = ["jan", "fév", "mars", "avr", "mai", "juin",
      "juil", "août", "sept", "oct", "nov", "déc"];
    const hh = String(d.getHours()).padStart(2, "0");
    const mm = String(d.getMinutes()).padStart(2, "0");
    return `${d.getDate()} ${months[d.getMonth()]}. · ${hh}h${mm}`;
  }
  return `Réveil ${time.replace(":", "h")}`;
}

const MODE_BADGE: Record<string, string> = {
  auto: "auto", explorer: "explorer", free: "free", evolve: "evolve",
};

export function HeartbeatList(props: HeartbeatListProps) {
  const {
    wakeups, selectedId, onSelect, onAdd,
    heartbeatActive, onToggleHeartbeat,
    stopAt, onStopAtChange,
    onContextMenu, activeSubTab, onSubTabChange,
  } = props;

  const lastStopAtRef = useRef(stopAt ?? "");
  if (stopAt) lastStopAtRef.current = stopAt;

  const handleStopAtToggle = useCallback(() => {
    if (stopAt !== null) {
      onStopAtChange(null);
    } else {
      const val = lastStopAtRef.current || defaultTomorrow();
      lastStopAtRef.current = val;
      onStopAtChange(val);
    }
  }, [stopAt, onStopAtChange]);

  return (
    <>
      <div className="list-header">
        <span className="list-title">Réveils</span>
        <div
          className={`toggle ${heartbeatActive ? "on" : ""}`}
          onClick={() => onToggleHeartbeat(!heartbeatActive)}
        />
      </div>
      <div className="stop-at-row">
        <span className="stop-at-label">Stop at</span>
        <div
          className={`toggle toggle-sm ${stopAt !== null ? "on" : ""}`}
          onClick={handleStopAtToggle}
        />
        {stopAt !== null && (
          <DatetimeInput
            value={stopAt}
            onChange={onStopAtChange}
            className="stop-at-input"
          />
        )}
      </div>
      <div className="list-tabs">
        <div
          className={`list-tab ${activeSubTab === "planned" ? "active" : ""}`}
          onClick={() => onSubTabChange("planned")}
        >Planifiés</div>
        <div
          className={`list-tab ${activeSubTab === "warning" ? "active" : ""}`}
          onClick={() => onSubTabChange("warning")}
        >Warning</div>
      </div>
      <div className="list-content">
        {wakeups.map((w) => (
          <div
            key={w.id}
            className={`list-item ${selectedId === w.id ? "active" : ""}`}
            onClick={() => onSelect(w.id)}
            onContextMenu={(e) => onContextMenu(e, w.id)}
          >
            <SignalDot state={w.active ? (props.sessionRunning ? "live" : "ok") : "idle"} />
            <div className="item-content">
              <div className="item-title">{formatWakeupTime(w.time)}</div>
              <div className="item-meta">
                --{w.mode} · {w.active ? "actif" : "inactif"}
              </div>
            </div>
            <div className={`item-badge ${MODE_BADGE[w.mode] ?? ""}`}>
              {w.mode}
            </div>
          </div>
        ))}
        <div className="list-add" onClick={onAdd}>+ Planifier un réveil</div>
      </div>
    </>
  );
}

function defaultTomorrow(): string {
  const t = new Date();
  t.setDate(t.getDate() + 1);
  return t.toISOString().slice(0, 16);
}

import { useCallback } from "react";
import type { ScheduledWakeup } from "@/types/config";
import { SignalDot, type SignalState } from "./signal-dot";
import "./heartbeat-list.css";

interface HeartbeatListProps {
  wakeups: ScheduledWakeup[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onAdd: () => void;
  heartbeatActive: boolean;
  onToggleHeartbeat: (active: boolean) => void;
  onContextMenu: (e: React.MouseEvent, id: string) => void;
  sessionSignal: SignalState;
  activeSubTab: "planned" | "warning";
  onSubTabChange: (tab: "planned" | "warning") => void;
}

const MODE_BADGE: Record<string, string> = {
  auto: "auto",
  explorer: "explorer",
  free: "free",
  evolve: "evolve",
};

export function HeartbeatList({
  wakeups,
  selectedId,
  onSelect,
  onAdd,
  heartbeatActive,
  onToggleHeartbeat,
  onContextMenu,
  sessionSignal,
  activeSubTab,
  onSubTabChange,
}: HeartbeatListProps) {
  const handleToggle = useCallback(() => {
    onToggleHeartbeat(!heartbeatActive);
  }, [heartbeatActive, onToggleHeartbeat]);

  return (
    <>
      <div className="list-header">
        <span className="list-title">Réveils</span>
        <div
          className={`toggle ${heartbeatActive ? "on" : ""}`}
          onClick={handleToggle}
        />
      </div>
      <div className="list-tabs">
        <div
          className={`list-tab ${activeSubTab.includes("planned") ? "active" : ""}`}
          onClick={() => onSubTabChange("planned")}
        >
          Planifiés
        </div>
        <div
          className={`list-tab ${activeSubTab.includes("warning") ? "active" : ""}`}
          onClick={() => onSubTabChange("warning")}
        >
          Warning
        </div>
      </div>
      <div className="list-content">
        {wakeups.map((w) => (
          <div
            key={w.id}
            className={`list-item ${selectedId?.includes(w.id) ? "active" : ""}`}
            onClick={() => onSelect(w.id)}
            onContextMenu={(e) => onContextMenu(e, w.id)}
          >
            <SignalDot state={w.active ? sessionSignal : "idle"} />
            <div className="item-content">
              <div className="item-title">Réveil {w.time}</div>
              <div className="item-meta">
                --{w.mode} · {w.active ? "planifié" : "inactif"}
              </div>
            </div>
            <div className={`item-badge ${MODE_BADGE[w.mode] ?? ""}`}>
              {w.mode}
            </div>
          </div>
        ))}
        <div className="list-add" onClick={onAdd}>
          + Planifier un réveil
        </div>
      </div>
    </>
  );
}

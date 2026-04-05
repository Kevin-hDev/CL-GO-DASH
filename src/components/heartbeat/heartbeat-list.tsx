import { useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import type { ScheduledWakeup } from "@/types/config";
import { DatetimeInput } from "@/components/ui/datetime-input";
import { Plus } from "@/components/ui/icons";
import { WakeupItem } from "./wakeup-item";

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
  renamingId?: string | null;
  onRename?: (id: string, name: string) => void;
  onCancelRename?: () => void;
}

export function HeartbeatList(props: HeartbeatListProps) {
  const { t } = useTranslation();
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
      {/* Header */}
      <div style={{
        display: "flex", alignItems: "center", justifyContent: "space-between",
        padding: "14px 16px", borderBottom: "1px solid var(--edge)",
      }}>
        <span style={{
          fontSize: "var(--text-sm)", fontWeight: 600,
          textTransform: "uppercase", letterSpacing: "0.5px", color: "var(--ink-muted)",
        }}>
          {t("heartbeat.wakeups")}
        </span>
        <div
          className={`toggle ${heartbeatActive ? "on" : ""}`}
          onClick={() => onToggleHeartbeat(!heartbeatActive)}
        />
      </div>

      {/* Stop at */}
      <div style={{
        display: "flex", alignItems: "center", gap: 8,
        padding: "10px 16px", borderBottom: "1px solid var(--edge)",
      }}>
        <span style={{
          fontSize: "var(--text-xs)", textTransform: "uppercase",
          letterSpacing: "0.5px", color: "var(--ink-faint)",
        }}>
          {t("heartbeat.stopAt")}
        </span>
        <div
          className={`toggle toggle-sm ${stopAt !== null ? "on" : ""}`}
          onClick={handleStopAtToggle}
        />
        {stopAt !== null && (
          <DatetimeInput value={stopAt} onChange={onStopAtChange} className="form-input" />
        )}
      </div>

      {/* Sub-tabs */}
      <div style={{
        display: "flex", gap: 6,
        padding: "10px 16px", borderBottom: "1px solid var(--edge)",
      }}>
        {(["planned", "warning"] as const).map((tab) => (
          <div
            key={tab}
            onClick={() => onSubTabChange(tab)}
            style={{
              padding: "6px 12px", fontSize: "var(--text-sm)",
              borderRadius: "var(--radius-sm)", cursor: "pointer",
              color: activeSubTab === tab ? "var(--pulse)" : "var(--ink-muted)",
              background: activeSubTab === tab ? "var(--pulse-muted)" : "transparent",
            }}
          >
            {t(`heartbeat.${tab}`)}
          </div>
        ))}
      </div>

      {/* Wakeup list — only when Planned tab is active */}
      {activeSubTab === "planned" && <div style={{ flex: 1, overflowY: "auto", padding: 8 }}>
        {wakeups.map((w) => (
          <WakeupItem
            key={w.id}
            wakeup={w}
            selected={selectedId === w.id}
            sessionRunning={props.sessionRunning}
            onSelect={onSelect}
            onContextMenu={onContextMenu}
            renaming={props.renamingId === w.id}
            onRename={props.onRename}
            onCancelRename={props.onCancelRename}
          />
        ))}
        <div
          onClick={onAdd}
          style={{
            display: "flex", alignItems: "center", justifyContent: "center",
            gap: 6, padding: "10px 12px", margin: 8,
            border: "1px dashed var(--edge)", borderRadius: "var(--radius-sm)",
            color: "var(--ink-faint)", fontSize: "var(--text-sm)", cursor: "pointer",
          }}
        >
          <Plus size={14} weight="bold" />
          {t("heartbeat.scheduleWakeup")}
        </div>
      </div>}
    </>
  );
}

function defaultTomorrow(): string {
  const t = new Date();
  t.setDate(t.getDate() + 1);
  return t.toISOString().slice(0, 16);
}

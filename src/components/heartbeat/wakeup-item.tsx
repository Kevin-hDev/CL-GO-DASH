import { useState, useRef, useEffect } from "react";
import type { ScheduledWakeup } from "@/types/config";
import { SignalDot } from "./signal-dot";
import { useKeyboard } from "@/hooks/use-keyboard";
import { cn } from "@/lib/utils";

const BADGE_STYLES: Record<string, React.CSSProperties> = {
  auto: { background: "var(--pulse-muted)", color: "var(--pulse)" },
  explorer: { background: "rgba(61,184,106,0.15)", color: "var(--signal-ok)" },
  free: { background: "rgba(226,184,66,0.15)", color: "var(--signal-live)" },
  evolve: { background: "rgba(217,68,68,0.12)", color: "var(--signal-error)" },
};

interface WakeupItemProps {
  wakeup: ScheduledWakeup;
  selected: boolean;
  sessionRunning: boolean;
  onSelect: (id: string) => void;
  onContextMenu: (e: React.MouseEvent, id: string) => void;
  renaming?: boolean;
  onRename?: (id: string, name: string) => void;
  onCancelRename?: () => void;
}

export function formatWakeupTime(time: string): string {
  if (time.includes("T")) {
    const d = new Date(time);
    const months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
      "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    const hh = String(d.getHours()).padStart(2, "0");
    const mm = String(d.getMinutes()).padStart(2, "0");
    return `${d.getDate()} ${months[d.getMonth()]}. · ${hh}h${mm}`;
  }
  return `Wakeup ${time}`;
}

export function WakeupItem({
  wakeup: w, selected, sessionRunning,
  onSelect, onContextMenu, renaming, onRename, onCancelRename,
}: WakeupItemProps) {
  return (
    <div
      onClick={() => onSelect(w.id)}
      onContextMenu={(e) => onContextMenu(e, w.id)}
      className={cn("transition-all duration-200 hover:translate-x-0.5")}
      style={{
        display: "flex", alignItems: "center", gap: 10,
        padding: "10px 12px", borderRadius: "var(--radius-sm)",
        cursor: "pointer", marginBottom: 2,
        background: selected ? "var(--pulse-muted)" : "transparent",
      }}
    >
      <SignalDot state={w.active ? (sessionRunning ? "live" : "ok") : "idle"} />
      <div style={{ flex: 1, minWidth: 0 }}>
        {renaming ? (
          <RenameInput
            defaultValue={w.name || formatWakeupTime(w.time)}
            onConfirm={(name) => onRename?.(w.id, name)}
            onCancel={() => onCancelRename?.()}
          />
        ) : (
          <div style={{
            fontSize: "var(--text-sm)", color: "var(--ink)",
            whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis",
          }}>
            {w.name || formatWakeupTime(w.time)}
          </div>
        )}
        <div style={{ fontSize: "var(--text-xs)", color: "var(--ink-faint)", marginTop: 2 }}>
          {w.name ? formatWakeupTime(w.time) + " · " : ""}--{w.mode} · {w.active ? "active" : "inactive"}
        </div>
      </div>
      <span style={{
        fontSize: "var(--text-xs)", padding: "2px 8px",
        borderRadius: "var(--radius-sm)", flexShrink: 0,
        ...(BADGE_STYLES[w.mode] ?? { background: "var(--surface)", color: "var(--ink-muted)" }),
      }}>
        {w.mode}
      </span>
    </div>
  );
}

function RenameInput({ defaultValue, onConfirm, onCancel }: {
  defaultValue: string;
  onConfirm: (name: string) => void;
  onCancel: () => void;
}) {
  const [val, setVal] = useState(defaultValue);
  const ref = useRef<HTMLInputElement>(null);

  useEffect(() => { ref.current?.focus(); ref.current?.select(); }, []);

  useKeyboard({
    onEscape: onCancel,
    onEnter: () => { if (val.trim()) onConfirm(val.trim()); },
  });

  return (
    <input
      ref={ref}
      className="rename-input"
      value={val}
      onChange={(e) => setVal(e.target.value)}
      onBlur={() => { if (val.trim()) onConfirm(val.trim()); else onCancel(); }}
      onClick={(e) => e.stopPropagation()}
    />
  );
}

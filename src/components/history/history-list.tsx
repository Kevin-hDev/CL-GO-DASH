import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import type { SessionMeta } from "@/types/session";
import { SignalDot } from "@/components/heartbeat/signal-dot";
import { Star } from "@phosphor-icons/react";
import { useKeyboard } from "@/hooks/use-keyboard";
import "./history-list.css";

type SubTab = "recent" | "archive" | "favorites";

interface HistoryListProps {
  items: SessionMeta[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  subTab: SubTab;
  onSubTabChange: (tab: SubTab) => void;
  onContextMenu: (e: React.MouseEvent, session: SessionMeta) => void;
  renamingId: string | null;
  onRename: (id: string, name: string) => void;
  onCancelRename: () => void;
  isFavorite: (id: string) => boolean;
}

const MODE_BADGE: Record<string, string> = {
  auto: "auto", explorer: "explorer", free: "free", evolve: "evolve",
};

function formatDuration(minutes: number): string {
  if (minutes < 1) return "<1min";
  if (minutes < 60) return `${Math.round(minutes)}min`;
  const h = Math.floor(minutes / 60);
  const m = Math.round(minutes % 60);
  return m > 0 ? `${h}h${m}` : `${h}h`;
}

function formatDate(iso: string): string {
  if (!iso) return "";
  const d = new Date(iso);
  const months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  return `${d.getDate()} ${months[d.getMonth()]}. · ${hh}h${mm}`;
}

function sessionTitle(s: SessionMeta): string {
  return s.custom_name || `Session ${formatDate(s.start).split(" ·")[0]}`;
}

export function HistoryList({
  items, selectedId, onSelect, subTab, onSubTabChange,
  onContextMenu, renamingId, onRename, onCancelRename, isFavorite,
}: HistoryListProps) {
  const { t } = useTranslation();

  const displayed = subTab === "favorites"
    ? items.filter((s) => isFavorite(s.id))
    : items;

  const tabs: { id: SubTab; label: string }[] = [
    { id: "recent", label: t("history.recent") },
    { id: "favorites", label: `★ ${t("history.favorites")}` },
    { id: "archive", label: t("history.archive") },
  ];

  return (
    <>
      <div className="hist-header">
        <span className="hist-title">Sessions</span>
      </div>
      <div className="hist-tabs">
        {tabs.map((tab) => (
          <div
            key={tab.id}
            className={`hist-tab ${subTab === tab.id ? "active" : ""}`}
            onClick={() => onSubTabChange(tab.id)}
          >
            {tab.label}
          </div>
        ))}
      </div>
      <div className="hist-content">
        {displayed.map((s) => (
          <div
            key={s.id}
            className={`hist-item ${selectedId === s.id ? "active" : ""}`}
            onClick={() => onSelect(s.id)}
            onContextMenu={(e) => onContextMenu(e, s)}
          >
            <SignalDot state={s.duration_minutes > 0 ? "ok" : "error"} />
            <div className="hist-item-body">
              {renamingId === s.id ? (
                <RenameInput
                  defaultValue={sessionTitle(s)}
                  onConfirm={(name) => onRename(s.id, name)}
                  onCancel={onCancelRename}
                />
              ) : (
                <div className="hist-item-name">
                  {isFavorite(s.id) && (
                    <Star size={12} weight="fill" style={{
                      color: "var(--signal-live)",
                      marginRight: 4,
                      verticalAlign: "middle",
                    }} />
                  )}
                  {sessionTitle(s)}
                </div>
              )}
              <div className="hist-item-meta">
                {formatDate(s.start)} · {formatDuration(s.duration_minutes)}
              </div>
            </div>
            <div className={`item-badge ${MODE_BADGE[s.mode] ?? ""}`}>
              {s.mode}
            </div>
          </div>
        ))}
        {displayed.length < 1 && (
          <div className="hist-empty">
            {subTab === "favorites" ? t("history.noFavorites") : t("history.noSessions")}
          </div>
        )}
      </div>
    </>
  );
}

function RenameInput({
  defaultValue, onConfirm, onCancel,
}: {
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
    />
  );
}

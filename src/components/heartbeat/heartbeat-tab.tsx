import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useWakeups } from "@/hooks/use-wakeups";
import { formatSchedule } from "@/lib/wakeup-format";
import { WakeupGrid } from "./wakeup-grid";
import { WakeupDetails } from "./wakeup-details";
import { NewWakeupDialog } from "./new-wakeup-dialog";
import type { ScheduledWakeup } from "@/types/wakeup";
import "./heartbeat.css";

type DialogState =
  | { kind: "none" }
  | { kind: "create" }
  | { kind: "edit"; wakeup: ScheduledWakeup };

export function HeartbeatTab(): { list: React.ReactNode; detail: React.ReactNode } {
  const { t } = useTranslation();
  const { wakeups, globalPaused, setPaused, toggle, remove, create, update } = useWakeups();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [dialog, setDialog] = useState<DialogState>({ kind: "none" });

  const selected = selectedId ? wakeups.find((w) => w.id === selectedId) ?? null : null;

  const activeWakeups = wakeups.filter((w) => w.active);

  const handleDelete = async () => {
    if (!selected) return;
    const id = selected.id;
    setSelectedId(null);
    await remove(id);
  };

  const list = (
    <div className="wk-sidebar">
      <div className="wk-sidebar-header">
        <span className="wk-sidebar-title">{t("heartbeat.sidebar.title")}</span>
        <button
          className="wk-sidebar-toggle"
          data-on={!globalPaused}
          onClick={() => setPaused(!globalPaused)}
          type="button"
          title={
            globalPaused
              ? t("heartbeat.sidebar.resume")
              : t("heartbeat.sidebar.pause")
          }
        >
          <span className="wk-toggle-dot" />
        </button>
      </div>
      <div className="wk-sidebar-list">
        {activeWakeups.length === 0 ? (
          <div className="wk-sidebar-empty">
            {globalPaused
              ? t("heartbeat.sidebar.pausedEmpty")
              : t("heartbeat.sidebar.empty")}
          </div>
        ) : (
          activeWakeups.map((w) => (
            <button
              key={w.id}
              className={`wk-sidebar-item ${selectedId === w.id ? "active" : ""}`}
              onClick={() => setSelectedId(w.id)}
              type="button"
            >
              <div className="wk-sidebar-model">{w.model}</div>
              <div className="wk-sidebar-schedule">{formatSchedule(w.schedule)}</div>
            </button>
          ))
        )}
      </div>
    </div>
  );

  const detail = (
    <>
      {selected ? (
        <WakeupDetails
          wakeup={selected}
          disableToggle={globalPaused}
          onBack={() => setSelectedId(null)}
          onToggle={(active) => toggle(selected.id, active)}
          onEdit={() => setDialog({ kind: "edit", wakeup: selected })}
          onDelete={handleDelete}
        />
      ) : (
        <WakeupGrid
          wakeups={wakeups}
          onSelect={setSelectedId}
          onCreate={() => setDialog({ kind: "create" })}
        />
      )}

      {dialog.kind !== "none" && (
        <NewWakeupDialog
          initial={dialog.kind === "edit" ? dialog.wakeup : null}
          onClose={() => setDialog({ kind: "none" })}
          onCreate={async (input) => { await create(input); }}
          onUpdate={async (w) => { await update(w); }}
        />
      )}
    </>
  );

  return { list, detail };
}

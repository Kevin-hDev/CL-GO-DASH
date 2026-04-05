import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { useHeartbeat } from "@/hooks/use-heartbeat";
import { useSessionStatus } from "@/hooks/use-session-status";
import { HeartbeatList } from "./heartbeat-list";
import { HeartbeatDetail } from "./heartbeat-detail";
import { Warnings } from "./warnings";
import { ContextMenu, type ContextMenuItem } from "@/components/ui/context-menu";
import { Pencil, Trash } from "@/components/ui/icons";

type SubTab = "planned" | "warning";

interface CtxState { x: number; y: number; id: string }

export function HeartbeatTab(): { list: React.ReactNode; detail: React.ReactNode } {
  const { t } = useTranslation();
  const hb = useHeartbeat();
  const sessionStatus = useSessionStatus();
  const [sub, setSub] = useState<SubTab>("planned");
  const [ctx, setCtx] = useState<CtxState | null>(null);
  const [renaming, setRenaming] = useState<string | null>(null);

  const onCtx = useCallback((e: React.MouseEvent, id: string) => {
    e.preventDefault();
    setCtx({ x: e.clientX, y: e.clientY, id });
  }, []);

  const ctxItems: ContextMenuItem[] = ctx ? [
    {
      label: t("history.rename"), icon: <Pencil size={14} />,
      onClick: () => { setRenaming(ctx.id); setCtx(null); },
    },
    {
      label: t("heartbeat.delete"), icon: <Trash size={14} />, danger: true,
      onClick: () => { hb.removeWakeup(ctx.id); setCtx(null); },
    },
  ] : [];

  const handleRename = useCallback((id: string, name: string) => {
    const w = hb.wakeups.find((x) => x.id === id);
    if (w) hb.saveWakeup({ ...w, name: name || null });
    setRenaming(null);
  }, [hb]);

  const list = (
    <>
      <HeartbeatList
        wakeups={hb.wakeups} selectedId={hb.selectedId}
        onSelect={hb.setSelectedId} onAdd={hb.addWakeup}
        heartbeatActive={hb.hbActive} onToggleHeartbeat={hb.toggleHeartbeat}
        stopAt={hb.stopAt} onStopAtChange={hb.setStopAt}
        sessionRunning={sessionStatus === "live"}
        onContextMenu={onCtx}
        activeSubTab={sub} onSubTabChange={setSub}
        renamingId={renaming}
        onRename={handleRename}
        onCancelRename={() => setRenaming(null)}
      />
      {ctx && <ContextMenu x={ctx.x} y={ctx.y} items={ctxItems} onClose={() => setCtx(null)} />}
    </>
  );

  let detail: React.ReactNode;
  if (sub === "warning") {
    detail = <Warnings />;
  } else if (!hb.selected) {
    detail = (
      <div style={{ padding: 24, color: "var(--ink-faint)" }}>
        {t("heartbeat.selectWakeup")}
      </div>
    );
  } else {
    detail = (
      <HeartbeatDetail
        wakeup={hb.selected}
        onSave={hb.saveWakeup}
        onToggleActive={hb.saveWakeup}
        onDelete={hb.removeWakeup}
        onRun={hb.runWakeup}
      />
    );
  }

  return { list, detail };
}

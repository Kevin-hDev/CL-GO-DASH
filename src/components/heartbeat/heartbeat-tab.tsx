import { useState, useCallback } from "react";
import { useHeartbeat } from "@/hooks/use-heartbeat";
import { useSessionStatus } from "@/hooks/use-session-status";
import { HeartbeatList } from "./heartbeat-list";
import { HeartbeatDetail } from "./heartbeat-detail";
import { Warnings } from "./warnings";
import { ContextMenu, type ContextMenuItem } from "@/components/ui/context-menu";

type SubTab = "planned" | "warning";

interface CtxState { x: number; y: number; id: string }

export function HeartbeatTab(): { list: React.ReactNode; detail: React.ReactNode } {
  const hb = useHeartbeat();
  const signal = useSessionStatus();
  const [sub, setSub] = useState<SubTab>("planned");
  const [ctx, setCtx] = useState<CtxState | null>(null);

  const onCtx = useCallback((e: React.MouseEvent, id: string) => {
    e.preventDefault();
    setCtx({ x: e.clientX, y: e.clientY, id });
  }, []);

  const ctxItems: ContextMenuItem[] = ctx ? [{
    label: "Supprimer", icon: "🗑", danger: true,
    onClick: () => { hb.removeWakeup(ctx.id); setCtx(null); },
  }] : [];

  const list = (
    <>
      <HeartbeatList
        wakeups={hb.wakeups} selectedId={hb.selectedId}
        onSelect={hb.setSelectedId} onAdd={hb.addWakeup}
        heartbeatActive={hb.hbActive} onToggleHeartbeat={hb.toggleHeartbeat}
        onContextMenu={onCtx} sessionSignal={signal}
        activeSubTab={sub} onSubTabChange={setSub}
      />
      {ctx && <ContextMenu x={ctx.x} y={ctx.y} items={ctxItems} onClose={() => setCtx(null)} />}
    </>
  );

  let detail: React.ReactNode;
  if (sub === "warning") {
    detail = <Warnings />;
  } else if (!hb.selected) {
    detail = <div style={{ padding: "var(--space-lg)", color: "var(--ink-faint)" }}>Sélectionne un réveil</div>;
  } else {
    detail = <HeartbeatDetail wakeup={hb.selected} onSave={hb.saveWakeup} onDelete={hb.removeWakeup} onRun={hb.runWakeup} />;
  }

  return { list, detail };
}

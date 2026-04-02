import { useState, useCallback } from "react";
import { useSessions } from "@/hooks/use-sessions";
import { HistoryList } from "./history-list";
import { SessionDetailView } from "./session-detail";
import { ContextMenu, type ContextMenuItem } from "@/components/ui/context-menu";
import type { SessionMeta } from "@/types/session";

interface CtxState { x: number; y: number; session: SessionMeta }

export function HistoryTab(): { list: React.ReactNode; detail: React.ReactNode } {
  const sess = useSessions();
  const [ctx, setCtx] = useState<CtxState | null>(null);

  const onCtx = useCallback((e: React.MouseEvent, session: SessionMeta) => {
    e.preventDefault();
    setCtx({ x: e.clientX, y: e.clientY, session });
  }, []);

  const ctxItems: ContextMenuItem[] = ctx ? [
    {
      label: "Supprimer", icon: "🗑", danger: true,
      onClick: () => {
        sess.deleteSession(ctx.session.file_path, ctx.session.id);
        setCtx(null);
      },
    },
  ] : [];

  const list = (
    <>
      <HistoryList
        items={sess.items}
        selectedId={sess.selectedId}
        onSelect={sess.loadDetail}
        subTab={sess.subTab}
        onSubTabChange={sess.setSubTab}
        onContextMenu={onCtx}
      />
      {ctx && <ContextMenu x={ctx.x} y={ctx.y} items={ctxItems} onClose={() => setCtx(null)} />}
    </>
  );

  let detail: React.ReactNode;
  if (sess.loading) {
    detail = <div style={{ padding: "var(--space-lg)", color: "var(--ink-faint)" }}>Chargement...</div>;
  } else if (!sess.detail) {
    detail = <div style={{ padding: "var(--space-lg)", color: "var(--ink-faint)" }}>Sélectionne une session</div>;
  } else {
    detail = <SessionDetailView detail={sess.detail} />;
  }

  return { list, detail };
}

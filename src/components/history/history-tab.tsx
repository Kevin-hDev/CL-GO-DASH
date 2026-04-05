import { useState, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useSessions } from "@/hooks/use-sessions";
import { useSessionStatus } from "@/hooks/use-session-status";
import { useFavorites } from "@/hooks/use-favorites";
import { HistoryList } from "./history-list";
import { SessionDetailView } from "./session-detail";
import { ContextMenu, type ContextMenuItem } from "@/components/ui/context-menu";
import { Pencil, Trash, Star } from "@phosphor-icons/react";
import type { SessionMeta } from "@/types/session";

interface CtxState { x: number; y: number; session: SessionMeta }

export function HistoryTab(): { list: React.ReactNode; detail: React.ReactNode } {
  const { t } = useTranslation();
  const sess = useSessions();
  const sessionStatus = useSessionStatus();
  const { isFavorite, toggleFavorite } = useFavorites();
  const [ctx, setCtx] = useState<CtxState | null>(null);
  const [renaming, setRenaming] = useState<string | null>(null);

  const isLive = sessionStatus === "live";

  useEffect(() => { sess.cleanup(); }, []); // eslint-disable-line

  const onCtx = useCallback((e: React.MouseEvent, session: SessionMeta) => {
    e.preventDefault();
    setCtx({ x: e.clientX, y: e.clientY, session });
  }, []);

  const ctxItems: ContextMenuItem[] = ctx ? [
    {
      label: t("history.rename"), icon: <Pencil size={14} />,
      onClick: () => { setRenaming(ctx.session.id); setCtx(null); },
    },
    {
      label: isFavorite(ctx.session.id) ? t("history.removeFavorite") : t("history.addFavorite"),
      icon: <Star size={14} weight={isFavorite(ctx.session.id) ? "fill" : "regular"} />,
      onClick: () => { toggleFavorite(ctx.session.id); setCtx(null); },
    },
    {
      label: t("history.delete"), icon: <Trash size={14} />, danger: true,
      onClick: () => { sess.deleteSession(ctx.session.file_path, ctx.session.id); setCtx(null); },
    },
  ] : [];

  const list = (
    <>
      <HistoryList
        items={sess.items} selectedId={sess.selectedId}
        onSelect={sess.loadDetail} subTab={sess.subTab}
        onSubTabChange={sess.setSubTab} onContextMenu={onCtx}
        renamingId={renaming}
        onRename={(id, name) => { sess.renameSession(id, name); setRenaming(null); }}
        onCancelRename={() => setRenaming(null)}
        isFavorite={isFavorite}
      />
      {ctx && <ContextMenu x={ctx.x} y={ctx.y} items={ctxItems} onClose={() => setCtx(null)} />}
    </>
  );

  let detail: React.ReactNode;
  if (sess.loading) {
    detail = <div style={{ padding: 24, color: "var(--ink-faint)" }}>{t("history.loading")}</div>;
  } else if (!sess.detail) {
    detail = <div style={{ padding: 24, color: "var(--ink-faint)" }}>{t("history.selectSession")}</div>;
  } else {
    detail = <SessionDetailView detail={sess.detail} isLive={isLive} />;
  }

  return { list, detail };
}

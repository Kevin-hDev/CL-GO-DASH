import { useState, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Plus, Pencil, Trash, ChatCircle } from "@/components/ui/icons";
import { ContextMenu, type ContextMenuItem } from "@/components/ui/context-menu";
import { useKeyboard } from "@/hooks/use-keyboard";
import type { AgentSessionMeta } from "@/types/agent";
import { idMatch } from "@/lib/utils";
import "./conversation.css";

interface ConversationListProps {
  sessions: AgentSessionMeta[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onCreate: () => void;
  onRename: (id: string, name: string) => void;
  onDelete: (id: string) => void;
}

interface CtxState { x: number; y: number; id: string }

export function ConversationList({
  sessions, selectedId, onSelect, onCreate, onRename, onDelete,
}: ConversationListProps) {
  const { t } = useTranslation();
  const [ctx, setCtx] = useState<CtxState | null>(null);
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const handleCtx = useCallback((e: React.MouseEvent, id: string) => {
    e.preventDefault();
    setCtx({ x: e.clientX, y: e.clientY, id });
  }, []);

  useKeyboard({
    onEscape: () => { setRenamingId(null); setCtx(null); },
  });

  const ctxItems: ContextMenuItem[] = ctx ? [
    {
      label: t("history.rename"), icon: <Pencil size={14} />,
      onClick: () => {
        setRenamingId(ctx.id);
        setTimeout(() => inputRef.current?.focus(), 0);
      },
    },
    {
      label: t("history.delete"), icon: <Trash size={14} />, danger: true,
      onClick: () => onDelete(ctx.id),
    },
  ] : [];

  const handleRenameSubmit = (id: string, value: string) => {
    if (value.trim()) onRename(id, value.trim());
    setRenamingId(null);
  };

  return (
    <>
      <div className="conv-header">
        <button className="conv-new-btn" onClick={onCreate}>
          <Plus size={14} weight="bold" />
          <span>{t("agentLocal.newSession")}</span>
        </button>
      </div>
      <div className="conv-list">
        {sessions.map((s) => {
          const active = idMatch(selectedId, s.id);
          const renaming = idMatch(renamingId, s.id);
          return (
            <div
              key={s.id}
              className={`conv-item ${active ? "active" : ""}`}
              onClick={() => onSelect(s.id)}
              onContextMenu={(e) => handleCtx(e, s.id)}
            >
              {renaming ? (
                <input
                  ref={inputRef}
                  className="conv-rename"
                  defaultValue={s.name}
                  onBlur={(e) => handleRenameSubmit(s.id, e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key.startsWith("Ent")) handleRenameSubmit(s.id, e.currentTarget.value);
                    if (e.key.startsWith("Esc")) setRenamingId(null);
                  }}
                />
              ) : (
                <>
                  <ChatCircle size={14} weight={active ? "fill" : "regular"} className="conv-icon" />
                  <span className="conv-name">{s.name}</span>
                </>
              )}
            </div>
          );
        })}
        {sessions.length < 1 && (
          <div className="hist-empty">{t("agentLocal.noConversations")}</div>
        )}
      </div>
      {ctx && (
        <ContextMenu x={ctx.x} y={ctx.y} items={ctxItems} onClose={() => setCtx(null)} />
      )}
    </>
  );
}

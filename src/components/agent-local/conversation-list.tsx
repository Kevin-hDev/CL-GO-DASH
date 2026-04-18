import { useState, useRef, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Pencil, Trash, ChatsCircle, CaretRight } from "@/components/ui/icons";
import { ComposeIcon } from "@/components/ui/compose-icon";
import { ContextMenu, type ContextMenuItem } from "@/components/ui/context-menu";
import { ProjectSection } from "./project-section";
import { useKeyboard } from "@/hooks/use-keyboard";
import { useProjectDrag } from "@/hooks/use-project-drag";
import type { AgentSessionMeta, Project } from "@/types/agent";
import { idMatch, displaySessionName } from "@/lib/utils";
import "./conversation.css";

interface ConversationListProps {
  sessions: AgentSessionMeta[];
  projects: Project[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onCreate: () => void;
  onRename: (id: string, name: string) => void;
  onDelete: (id: string) => void;
  onNewSessionInProject: (projectId: string) => void;
  onRenameProject: (id: string, name: string) => void;
  onDeleteProject: (id: string) => void;
  onOpenFolder: (path: string) => void;
  onReorderProjects: (ids: string[]) => void;
}

export function ConversationList({
  sessions, projects, selectedId,
  onSelect, onCreate, onRename, onDelete,
  onNewSessionInProject, onRenameProject, onDeleteProject,
  onOpenFolder, onReorderProjects,
}: ConversationListProps) {
  const { t } = useTranslation();
  const [ctx, setCtx] = useState<{ x: number; y: number; id: string } | null>(null);
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const ghostRef = useRef<HTMLDivElement | null>(null);
  const [projectsCollapsed, setProjectsCollapsed] = useState(false);
  const [collapsedProjects, setCollapsedProjects] = useState<Set<string>>(new Set());
  const [discussionsCollapsed, setDiscussionsCollapsed] = useState(false);

  const toggleProject = useCallback((id: string) => {
    setCollapsedProjects((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  }, []);

  const projectIds = projects.map((p) => p.id);
  const drag = useProjectDrag(projectIds, onReorderProjects);

  useKeyboard({
    onEscape: () => { setRenamingId(null); setCtx(null); drag.onCancel(); },
  });

  useEffect(() => {
    if (!drag.draggingId) {
      if (ghostRef.current) { ghostRef.current.remove(); ghostRef.current = null; }
      return;
    }
    const srcEl = document.querySelector(`[data-project-id="${drag.draggingId}"] .conv-project-header`);
    if (!srcEl) return;
    const ghost = document.createElement("div");
    ghost.className = "conv-drag-ghost";
    ghost.textContent = srcEl.textContent;
    document.body.appendChild(ghost);
    ghostRef.current = ghost;

    const onMove = (e: PointerEvent) => {
      ghost.style.left = `${e.clientX + 12}px`;
      ghost.style.top = `${e.clientY - 12}px`;
      const el = document.elementFromPoint(e.clientX, e.clientY);
      const wrapper = el?.closest("[data-project-id]");
      const id = wrapper?.getAttribute("data-project-id");
      if (id) drag.onHover(id);
    };
    const onUp = () => drag.onRelease();

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      if (ghostRef.current) { ghostRef.current.remove(); ghostRef.current = null; }
    };
  }, [drag.draggingId, drag.onHover, drag.onRelease]);

  const handleCtx = useCallback((e: React.MouseEvent, id: string) => {
    e.preventDefault();
    setCtx({ x: e.clientX, y: e.clientY, id });
  }, []);

  const ctxItems: ContextMenuItem[] = ctx ? [
    { label: t("history.rename"), icon: <Pencil size={14} />, onClick: () => { setRenamingId(ctx.id); setTimeout(() => inputRef.current?.focus(), 0); } },
    { label: t("history.delete"), icon: <Trash size={14} />, danger: true, onClick: () => onDelete(ctx.id) },
  ] : [];

  const handleRenameSubmit = (id: string, value: string) => {
    if (value.trim()) onRename(id, value.trim());
    setRenamingId(null);
  };

  const displayOrder = drag.liveOrder ?? projectIds;
  const projectMap = new Map(projects.map((p) => [p.id, p]));
  const projectIdSet = new Set(projectIds);
  const orphanSessions = sessions.filter(
    (s) => !s.project_id || !projectIdSet.has(s.project_id),
  );

  return (
    <>
      <div className="conv-header">
        <button className="conv-new-btn" onClick={onCreate}>
          <ComposeIcon size={14} />
          <span>{t("agentLocal.newSession")}</span>
        </button>
      </div>
      <div className={`conv-list ${drag.draggingId ? "is-dragging" : ""}`}>
        {projects.length > 0 && (
          <>
            <div
              className="conv-section-label conv-section-toggle"
              onClick={() => setProjectsCollapsed((c) => !c)}
            >
              <CaretRight size={10} className={`conv-collapse-chevron ${projectsCollapsed ? "" : "conv-collapse-open"}`} />
              {t("projects.title", "Projets")}
            </div>
            {!projectsCollapsed && displayOrder.map((id) => {
              const p = projectMap.get(id);
              if (!p) return null;
              return (
                <ProjectSection
                  key={p.id}
                  project={p}
                  sessions={sessions.filter((s) => s.project_id === p.id)}
                  selectedId={selectedId}
                  isDragOver={false}
                  isDragging={drag.draggingId === p.id}
                  onSelect={onSelect}
                  onNewSession={onNewSessionInProject}
                  onRenameProject={onRenameProject}
                  onDeleteProject={onDeleteProject}
                  onOpenFolder={onOpenFolder}
                  onRenameSession={onRename}
                  onDeleteSession={onDelete}
                  onGrab={drag.onGrab}
                  collapsed={collapsedProjects.has(p.id)}
                  onToggleCollapse={() => toggleProject(p.id)}
                />
              );
            })}
          </>
        )}

        {orphanSessions.length > 0 && (
          <>
            {projects.length > 0 && (
              <div
                className="conv-section-label conv-section-toggle"
                onClick={() => setDiscussionsCollapsed((c) => !c)}
              >
                <CaretRight size={10} className={`conv-collapse-chevron ${discussionsCollapsed ? "" : "conv-collapse-open"}`} />
                {t("projects.discussions", "Discussions")}
              </div>
            )}
            {!discussionsCollapsed && orphanSessions.map((s) => {
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
                      <ChatsCircle size={14} weight={active ? "fill" : "regular"} className="conv-icon" />
                      <span className="conv-name">{displaySessionName(s.name, t)}</span>
                    </>
                  )}
                </div>
              );
            })}
          </>
        )}

        {sessions.length === 0 && projects.length === 0 && (
          <div className="hist-empty">{t("agentLocal.noConversations")}</div>
        )}
      </div>
      {ctx && <ContextMenu x={ctx.x} y={ctx.y} items={ctxItems} onClose={() => setCtx(null)} />}
    </>
  );
}

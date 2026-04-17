import { useState, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  FolderSimple, FolderOpen, DotsThreeVertical, PencilSimple,
  X, ChatCircle, Plus, CaretRight,
} from "@/components/ui/icons";
import { ContextMenu, type ContextMenuItem } from "@/components/ui/context-menu";
import { useKeyboard } from "@/hooks/use-keyboard";
import type { AgentSessionMeta, Project } from "@/types/agent";
import { idMatch, displaySessionName } from "@/lib/utils";

interface ProjectSectionProps {
  project: Project;
  sessions: AgentSessionMeta[];
  selectedId: string | null;
  isDragOver: boolean;
  onSelect: (id: string) => void;
  onNewSession: (projectId: string) => void;
  onRenameProject: (id: string, name: string) => void;
  onDeleteProject: (id: string) => void;
  onOpenFolder: (path: string) => void;
  onRenameSession: (id: string, name: string) => void;
  onDeleteSession: (id: string) => void;
  onGrab: (id: string) => void;
  isDragging: boolean;
  collapsed: boolean;
  onToggleCollapse: () => void;
}

export function ProjectSection({
  project, sessions, selectedId, isDragOver,
  onSelect, onNewSession, onRenameProject, onDeleteProject,
  onOpenFolder, onRenameSession, onDeleteSession,
  onGrab, isDragging, collapsed, onToggleCollapse,
}: ProjectSectionProps) {
  const { t } = useTranslation();
  const [ctx, setCtx] = useState<{ x: number; y: number } | null>(null);
  const [renaming, setRenaming] = useState(false);
  const [sessionCtx, setSessionCtx] = useState<{ x: number; y: number; id: string } | null>(null);
  const [renamingSessionId, setRenamingSessionId] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const sessionInputRef = useRef<HTMLInputElement>(null);

  useKeyboard({
    onEscape: () => { setCtx(null); setRenaming(false); setSessionCtx(null); setRenamingSessionId(null); },
  });

  const projectMenuItems: ContextMenuItem[] = [
    { label: t("projects.openFolder", "Ouvrir le dossier"), icon: <FolderOpen size={14} />, onClick: () => onOpenFolder(project.path) },
    { label: t("projects.rename", "Renommer"), icon: <PencilSimple size={14} />, onClick: () => { setRenaming(true); setTimeout(() => inputRef.current?.focus(), 0); } },
    { label: t("projects.delete", "Supprimer"), icon: <X size={14} />, onClick: () => onDeleteProject(project.id) },
  ];

  const sessionMenuItems: ContextMenuItem[] = sessionCtx ? [
    { label: t("history.rename"), icon: <PencilSimple size={14} />, onClick: () => { setRenamingSessionId(sessionCtx.id); setTimeout(() => sessionInputRef.current?.focus(), 0); } },
    { label: t("history.delete"), icon: <X size={14} />, onClick: () => onDeleteSession(sessionCtx.id) },
  ] : [];

  const handleRename = useCallback((value: string) => {
    if (value.trim()) onRenameProject(project.id, value.trim());
    setRenaming(false);
  }, [project.id, onRenameProject]);

  const handleSessionRename = useCallback((id: string, value: string) => {
    if (value.trim()) onRenameSession(id, value.trim());
    setRenamingSessionId(null);
  }, [onRenameSession]);

  return (
    <div
      className={`conv-project-wrapper ${isDragOver ? "conv-project-drag-over" : ""} ${isDragging ? "conv-project-dragging" : ""}`}
      data-project-id={project.id}
    >
      <div
        className="conv-project-header"
        style={{ cursor: isDragging ? "grabbing" : "grab" }}
        onClick={onToggleCollapse}
        onPointerDown={(e) => {
          if (e.button !== 0) return;
          e.preventDefault();
          onGrab(project.id);
        }}
      >
        {renaming ? (
          <input
            ref={inputRef}
            className="conv-rename"
            defaultValue={project.name}
            onClick={(e) => e.stopPropagation()}
            onBlur={(e) => handleRename(e.target.value)}
            onKeyDown={(e) => {
              if (e.key.startsWith("Ent")) handleRename(e.currentTarget.value);
              if (e.key.startsWith("Esc")) setRenaming(false);
            }}
          />
        ) : (
          <>
            <CaretRight size={10} className={`conv-collapse-chevron ${collapsed ? "" : "conv-collapse-open"}`} />
            <FolderSimple size={14} className="conv-icon" />
            <span className="conv-project-name">{project.name}</span>
            <div className="conv-project-actions">
              <button className="conv-project-action-btn" onClick={(e) => { e.stopPropagation(); setCtx({ x: e.clientX, y: e.clientY }); }}>
                <DotsThreeVertical size={14} />
              </button>
              <button className="conv-project-action-btn" onClick={(e) => { e.stopPropagation(); onNewSession(project.id); }}>
                <Plus size={12} weight="bold" />
              </button>
            </div>
          </>
        )}
      </div>

      {!collapsed && sessions.map((s) => {
        const active = idMatch(selectedId, s.id);
        const isRenaming = idMatch(renamingSessionId, s.id);
        return (
          <div
            key={s.id}
            className={`conv-item conv-session-indented ${active ? "active" : ""}`}
            onClick={() => onSelect(s.id)}
            onContextMenu={(e) => { e.preventDefault(); setSessionCtx({ x: e.clientX, y: e.clientY, id: s.id }); }}
          >
            {isRenaming ? (
              <input
                ref={sessionInputRef}
                className="conv-rename"
                defaultValue={s.name}
                onBlur={(e) => handleSessionRename(s.id, e.target.value)}
                onKeyDown={(e) => {
                  if (e.key.startsWith("Ent")) handleSessionRename(s.id, e.currentTarget.value);
                  if (e.key.startsWith("Esc")) setRenamingSessionId(null);
                }}
              />
            ) : (
              <>
                <ChatCircle size={14} weight={active ? "fill" : "regular"} className="conv-icon" />
                <span className="conv-name">{displaySessionName(s.name, t)}</span>
              </>
            )}
          </div>
        );
      })}

      {!collapsed && sessions.length === 0 && (
        <div className="conv-session-indented" style={{ color: "var(--ink-faint)", fontSize: "var(--text-xs)", padding: "var(--space-xs) 0" }}>
          {t("projects.noDiscussion", "Aucune discussion")}
        </div>
      )}

      {ctx && <ContextMenu x={ctx.x} y={ctx.y} items={projectMenuItems} onClose={() => setCtx(null)} />}
      {sessionCtx && <ContextMenu x={sessionCtx.x} y={sessionCtx.y} items={sessionMenuItems} onClose={() => setSessionCtx(null)} />}
    </div>
  );
}

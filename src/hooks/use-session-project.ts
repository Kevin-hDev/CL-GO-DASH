import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import type { AgentSession, Project } from "@/types/agent";

export function useSessionProject(
  sessionId: string,
  projects: Project[],
  onAddProject: (path: string) => Promise<Project>,
  hasMessages: boolean,
) {
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);

  useEffect(() => {
    invoke<AgentSession>("get_agent_session", { id: sessionId })
      .then((s) => setSelectedProjectId(s.project_id ?? null))
      .catch((e) => console.warn("Session project load:", e));
  }, [sessionId]);

  const selectedProject = projects.find((p) => p.id === selectedProjectId);
  const locked = hasMessages && !!selectedProjectId;
  const hidden = hasMessages && !selectedProjectId;

  const handleAddProject = useCallback(async () => {
    const result = await openFileDialog({ directory: true });
    if (!result) return;
    const path = typeof result === "string" ? result : String(result);
    const project = await onAddProject(path);
    setSelectedProjectId(project.id);
  }, [onAddProject]);

  return {
    selectedProjectId,
    setSelectedProjectId,
    selectedProject,
    locked,
    hidden,
    handleAddProject,
  };
}

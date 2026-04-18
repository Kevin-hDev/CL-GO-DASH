import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { ChatInput } from "./chat-input";
import { ProjectSelector } from "./project-selector";
import { usePermissionMode } from "@/hooks/use-permission-mode";
import type { Project } from "@/types/agent";
import type { DroppedFile } from "@/hooks/use-file-drop";
import "./welcome-view.css";

interface WelcomeViewProps {
  model: string;
  provider: string;
  projects: Project[];
  onAddProject: (path: string) => Promise<Project>;
  onSend: (text: string, projectId?: string) => void;
  onModelChange: (model: string, provider: string) => void;
}

export function WelcomeView({
  model, provider, projects, onAddProject, onSend, onModelChange,
}: WelcomeViewProps) {
  const { t } = useTranslation();
  const permMode = usePermissionMode();
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);
  const [leaving, setLeaving] = useState(false);

  const handleAddProject = useCallback(async () => {
    const result = await openFileDialog({ directory: true });
    if (!result) return;
    const path = typeof result === "string" ? result : String(result);
    const project = await onAddProject(path);
    setSelectedProjectId(project.id);
  }, [onAddProject]);

  const handleSend = useCallback((text: string, _files?: DroppedFile[]) => {
    if (!text.trim()) return;
    setLeaving(true);
    setTimeout(() => {
      onSend(text, selectedProjectId ?? undefined);
    }, 350);
  }, [onSend, selectedProjectId]);

  return (
    <div className={`welcome-zone ${leaving ? "welcome-leaving" : ""}`}>
      <div className="welcome-content">
        <h1 className={`welcome-title ${leaving ? "welcome-title-leave" : ""}`}>
          {t("welcome.title")}
        </h1>
        <div className={`welcome-input-wrap ${leaving ? "welcome-input-leave" : ""}`}>
          <ChatInput
            modelName={model}
            providerName={provider}
            isStreaming={false}
            thinkingEnabled={false}
            contextUsed={0}
            contextMax={0}
            permissionMode={permMode.mode}
            onPermissionModeChange={permMode.change}
            onSend={handleSend}
            onStop={() => {}}
            onFileImport={() => {}}
            onModelChange={onModelChange}
            onToggleThinking={() => {}}
          />
          <ProjectSelector
            projects={projects}
            selectedProjectId={selectedProjectId}
            locked={false}
            hidden={false}
            onSelect={setSelectedProjectId}
            onAddProject={handleAddProject}
          />
        </div>
      </div>
    </div>
  );
}

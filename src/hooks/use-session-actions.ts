import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import type { AgentSessionMeta, Project } from "@/types/agent";
import type { DroppedFile } from "@/hooks/use-file-drop";

interface TabStateRef {
  deselectTab: () => void;
  addTab: (sessionId: string, label: string) => Promise<void>;
  activeIndex: number;
  renameTab: (index: number, label: string) => Promise<void>;
}

interface ProjectsHookRef {
  projects: Project[];
}

interface CreateFn {
  (name: string, model: string, provider?: string, projectId?: string): Promise<AgentSessionMeta>;
}

interface RenameFn {
  (id: string, name: string): Promise<void>;
}

export interface SessionActionsDeps {
  create: CreateFn;
  tabState: TabStateRef;
  rename: RenameFn;
  defaultModel: string;
  defaultProvider: string;
  welcomeModel: { model: string; provider: string } | null;
  setWelcomeModel: (v: { model: string; provider: string } | null) => void;
  projectsHook: ProjectsHookRef;
}

export function useSessionActions(deps: SessionActionsDeps) {
  const { t } = useTranslation();
  const {
    create,
    tabState,
    rename,
    defaultModel,
    defaultProvider,
    welcomeModel,
    setWelcomeModel,
    projectsHook,
  } = deps;

  const [pendingMessage, setPendingMessage] = useState<string | null>(null);
  const [pendingWorkingDir, setPendingWorkingDir] = useState<string | undefined>(undefined);
  const [pendingSkills, setPendingSkills] = useState<{ name: string; content: string }[] | undefined>(undefined);
  const [pendingFiles, setPendingFiles] = useState<DroppedFile[] | undefined>(undefined);

  const handleCreate = useCallback(() => {
    tabState.deselectTab();
  }, [tabState]);

  const handleCreateWithModel = useCallback(
    async (newModel: string, newProvider: string) => {
      const name = t("agentLocal.newSession");
      const session = await create(name, newModel, newProvider);
      await tabState.addTab(session.id, session.name);
    },
    [create, tabState, t],
  );

  const handleWelcomeSend = useCallback(
    async (text: string, files?: DroppedFile[], projectId?: string, skills?: { name: string; content: string }[]) => {
      const name = text.slice(0, 40).trim() || t("agentLocal.newSession");
      const m = welcomeModel ?? { model: defaultModel, provider: defaultProvider };
      const project = projectId ? projectsHook.projects.find((p) => p.id === projectId) : undefined;
      const session = await create(name, m.model, m.provider, projectId);
      setPendingMessage(text);
      setPendingWorkingDir(project?.path);
      setPendingSkills(skills);
      setPendingFiles(files && files.length > 0 ? files : undefined);
      setWelcomeModel(null);
      await tabState.addTab(session.id, session.name);
    },
    [create, tabState, defaultModel, defaultProvider, welcomeModel, setWelcomeModel, t, projectsHook.projects],
  );

  const handleAutoRename = useCallback(
    async (id: string, name: string) => {
      await rename(id, name);
      if (tabState.activeIndex >= 0) {
        await tabState.renameTab(tabState.activeIndex, name);
      }
    },
    [rename, tabState],
  );

  const handleCreateInProject = useCallback(
    async (projectId: string) => {
      const name = t("agentLocal.newSession");
      const session = await create(name, defaultModel, defaultProvider, projectId);
      await tabState.addTab(session.id, session.name);
    },
    [create, tabState, defaultModel, defaultProvider, t],
  );

  return {
    pendingMessage,
    setPendingMessage,
    pendingWorkingDir,
    setPendingWorkingDir,
    pendingSkills,
    setPendingSkills,
    pendingFiles,
    setPendingFiles,
    handleCreate,
    handleCreateWithModel,
    handleWelcomeSend,
    handleAutoRename,
    handleCreateInProject,
  };
}

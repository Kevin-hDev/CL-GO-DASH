import { useState, useCallback, useEffect, useRef, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAgentSessions } from "@/hooks/use-agent-sessions";
import { useAgentTabs } from "@/hooks/use-agent-tabs";
import { useProjects } from "@/hooks/use-projects";
import { useTerminal } from "@/hooks/use-terminal";
import { useDefaultModel } from "@/hooks/use-default-model";
import { useAvailableModels } from "@/hooks/use-available-models";
import { useSessionActions } from "@/hooks/use-session-actions";
import { useFilePreview } from "@/hooks/use-file-preview";
import { useAgentLocalShortcuts } from "@/hooks/use-agent-local-shortcuts";
import { useArrowNavigation } from "@/hooks/use-arrow-navigation";
import type { FileOperation } from "@/types/file-preview";
import type { AgentLocalNavState, DeepPartial } from "@/types/navigation";

interface UseAgentLocalTabOpts {
  navState: AgentLocalNavState;
  onSessionChange?: (id: string | null) => void;
  onNavChange?: (partial: DeepPartial<AgentLocalNavState>) => void;
  onNavReplace?: (partial: DeepPartial<AgentLocalNavState>) => void;
  listFocused: boolean;
}

export function useAgentLocalTab({ navState, onSessionChange, onNavChange, onNavReplace, listFocused }: UseAgentLocalTabOpts) {
  const { sessions, refresh, create, rename, remove, updateModel } = useAgentSessions();
  const tabState = useAgentTabs();
  const projectsHook = useProjects();

  const activeSession = tabState.activeSessionId
    ? sessions.find((s) => s.id.localeCompare(tabState.activeSessionId) === 0)
    : null;
  const activeProject = activeSession?.project_id
    ? projectsHook.projects.find((p) => p.id === activeSession.project_id)
    : null;
  const terminalGroupKey = activeProject?.id || "__default__";
  const terminalCwd = activeProject?.path || "";
  const validGroupKeys = projectsHook.projects.map((p) => p.id);
  const terminal = useTerminal(terminalGroupKey, terminalCwd, validGroupKeys);
  const { model: defaultModel, provider: defaultProvider } = useDefaultModel();
  const [welcomeModel, setWelcomeModel] = useState<{ model: string; provider: string } | null>(null);
  const [thinking, setThinking] = useState(false);
  const [fileOperations, setFileOperations] = useState<FileOperation[]>([]);
  const reportedPanelState = useRef(false);
  const restoredPanelNavKey = useRef<string | null>(null);

  const { groups: availableModels } = useAvailableModels();

  const currentDefault = welcomeModel ?? { model: defaultModel, provider: defaultProvider };
  const model = activeSession?.model ?? currentDefault.model;
  const provider = activeSession?.provider ?? currentDefault.provider;
  const filePreview = useFilePreview(tabState.activeSessionId ?? null, fileOperations);

  useEffect(() => {
    if (!model || availableModels.size === 0) return;
    const providerModels = availableModels.get(provider);
    if (!providerModels) return;
    const stillExists = providerModels.some((m) => m.id === model);
    if (stillExists) return;
    const allModels = Array.from(availableModels.values()).flat();
    if (allModels.length === 0) return;
    const fallback = allModels[0];
    if (tabState.activeSessionId) {
      void updateModel(tabState.activeSessionId, fallback.id, fallback.provider_id);
    } else {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- fallback when selected model is removed
      setWelcomeModel({ model: fallback.id, provider: fallback.provider_id });
    }
  }, [availableModels, model, provider, tabState.activeSessionId, updateModel]);

  const sessionActions = useSessionActions({ create, tabState, rename, defaultModel, defaultProvider, welcomeModel, setWelcomeModel, projectsHook });

  useAgentLocalShortcuts({
    activeSessionId: tabState.activeSessionId,
    terminalOpen: terminal.isOpen,
    terminalTabsCount: terminal.tabs.length,
    terminalCwd,
    onAddTerminalTab: terminal.addTab,
    onToggleTerminal: terminal.togglePanel,
  });

  const prevSessionRef = useRef(tabState.activeSessionId);
  useEffect(() => {
    if (tabState.activeSessionId !== prevSessionRef.current) {
      prevSessionRef.current = tabState.activeSessionId;
      onSessionChange?.(tabState.activeSessionId ?? null);
    }
  }, [tabState.activeSessionId, onSessionChange]);

  const handleSelectById = useCallback(async (id: string) => {
    const existingIdx = tabState.tabs.findIndex((tab) => tab.session_id.localeCompare(id) === 0);
    if (existingIdx >= 0) {
      await tabState.selectTab(existingIdx);
      return;
    }
    const label = sessions.find((s) => s.id.localeCompare(id) === 0)?.name ?? "Chat";
    if (tabState.activeIndex >= 0 && tabState.activeIndex < tabState.tabs.length) {
      await tabState.updateTab(tabState.activeIndex, id, label);
    } else {
      await tabState.addTab(id, label);
    }
  }, [tabState, sessions]);

  useEffect(() => {
    if (navState.sessionId === tabState.activeSessionId) return;
    if (navState.sessionId === null) {
      void tabState.deselectTab();
    } else {
      void handleSelectById(navState.sessionId);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- only react to requested session changes
  }, [navState.sessionId]);

  const panelNavKey = JSON.stringify([
    navState.previewOpen,
    navState.previewActiveTab,
    navState.previewFullscreen,
    navState.terminalOpen,
    navState.terminalActiveTabId,
  ]);

  useEffect(() => {
    if (restoredPanelNavKey.current === panelNavKey) return;
    restoredPanelNavKey.current = panelNavKey;
    if (filePreview.open !== navState.previewOpen) filePreview.setOpen(navState.previewOpen);
    if (filePreview.activeTab !== navState.previewActiveTab) filePreview.setActiveTab(navState.previewActiveTab);
    if (filePreview.fullscreen !== navState.previewFullscreen) filePreview.setFullscreen(navState.previewFullscreen);
    if (terminal.isOpen !== navState.terminalOpen) terminal.togglePanel();
    if (navState.terminalActiveTabId && terminal.activeTabId !== navState.terminalActiveTabId) {
      terminal.setActiveTab(navState.terminalActiveTabId);
    }
  }, [
    panelNavKey,
    navState.previewOpen, navState.previewActiveTab, navState.previewFullscreen,
    navState.terminalOpen, navState.terminalActiveTabId,
    filePreview, terminal,
  ]);

  useEffect(() => {
    const report = reportedPanelState.current ? onNavChange : onNavReplace ?? onNavChange;
    reportedPanelState.current = true;
    report?.({
      previewOpen: filePreview.open,
      previewActiveTab: filePreview.activeTab,
      previewFullscreen: filePreview.fullscreen,
      terminalOpen: terminal.isOpen,
      terminalActiveTabId: terminal.activeTabId ?? null,
    });
  }, [
    filePreview.open, filePreview.activeTab, filePreview.fullscreen,
    terminal.isOpen, terminal.activeTabId, onNavChange, onNavReplace,
  ]);

  const visibleSessionIds = useMemo(() => {
    const projectIdSet = new Set(projectsHook.projects.map((p) => p.id));
    const byProject = projectsHook.projects.flatMap((p) =>
      sessions.filter((s) => s.project_id === p.id).map((s) => s.id),
    );
    const orphans = sessions
      .filter((s) => !s.project_id || !projectIdSet.has(s.project_id))
      .map((s) => s.id);
    return [...byProject, ...orphans];
  }, [sessions, projectsHook.projects]);

  useArrowNavigation({
    items: visibleSessionIds,
    selectedId: tabState.activeSessionId,
    onSelect: (id) => void handleSelectById(id),
    enabled: listFocused,
  });

  const handleDeleteProject = useCallback((projectId: string) => {
    const entries = terminal.getGroupPtyEntries(projectId);
    for (const { id, token } of entries) {
      invoke("pty_kill", { id, token }).catch(() => {});
    }
    terminal.removeGroup(projectId);
    void projectsHook.remove(projectId);
  }, [terminal, projectsHook]);

  return {
    sessions, refresh, rename, remove, updateModel,
    tabState, projectsHook, terminal, activeSession,
    model, provider, currentDefault, activeProject,
    filePreview, fileOperations, setFileOperations,
    thinking, setThinking, welcomeModel, setWelcomeModel,
    sessionActions, handleSelectById, handleDeleteProject,
  };
}

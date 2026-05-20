import { useState, useCallback, useEffect, useMemo } from "react";
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
import { useAgentLocalTabPanelSync } from "@/hooks/use-agent-local-tab-panel-sync";
import { useAgentLocalControlledPreview } from "@/hooks/use-agent-local-controlled-preview";
import { useAgentLocalControlledTerminal } from "@/hooks/use-agent-local-controlled-terminal";
import { useArrowNavigation } from "@/hooks/use-arrow-navigation";
import type { FileOperation } from "@/types/file-preview";
import type { AgentLocalNavState, DeepPartial } from "@/types/navigation";
import {
  normalizeReasoningMode,
  reasoningModeOptions,
  type ReasoningMode,
} from "@/lib/reasoning-modes";

interface UseAgentLocalTabOpts {
  navState: AgentLocalNavState;
  onSessionChange?: (id: string | null) => void;
  onNavChange?: (partial: DeepPartial<AgentLocalNavState>) => void;
  listFocused: boolean;
}

export function useAgentLocalTab({ navState, onSessionChange, onNavChange, listFocused }: UseAgentLocalTabOpts) {
  const { sessions, refresh, create, rename, remove, updateModel, updateReasoning } = useAgentSessions();
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
  const terminalState = useTerminal(terminalGroupKey, terminalCwd, validGroupKeys);
  const { model: defaultModel, provider: defaultProvider } = useDefaultModel();
  const [welcomeModel, setWelcomeModel] = useState<{ model: string; provider: string } | null>(null);
  const [welcomeReasoningMode, setWelcomeReasoningMode] = useState<string | null>(null);
  const [fileOperations, setFileOperations] = useState<FileOperation[]>([]);

  const { groups: availableModels } = useAvailableModels();

  const normalizeForSelectedModel = useCallback(
    (nextModel: string, nextProvider: string, currentMode: string | null | undefined) => {
      const entry = availableModels.get(nextProvider)?.find((m) => m.id === nextModel) ?? null;
      const options = reasoningModeOptions(entry);
      return {
        mode: normalizeReasoningMode(currentMode, options),
        supportsThinking: entry?.supports_thinking ?? false,
      };
    },
    [availableModels],
  );

  const currentDefault = welcomeModel ?? { model: defaultModel, provider: defaultProvider };
  const model = activeSession?.model ?? currentDefault.model;
  const provider = activeSession?.provider ?? currentDefault.provider;
  const reasoningMode = activeSession
    ? activeSession.reasoning_mode ?? (activeSession.thinking_enabled ? "auto" : null)
    : welcomeReasoningMode;
  const filePreviewState = useFilePreview(tabState.activeSessionId ?? null, fileOperations);

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

  const sessionActions = useSessionActions({
    create,
    tabState,
    rename,
    defaultModel,
    defaultProvider,
    welcomeModel,
    setWelcomeModel,
    welcomeReasoningMode,
    projectsHook,
    onSessionChange,
  });

  const setReasoningMode = useCallback((mode: ReasoningMode) => {
    const currentSupportsThinking = normalizeForSelectedModel(model, provider, mode).supportsThinking;
    if (tabState.activeSessionId) {
      void updateReasoning(tabState.activeSessionId, mode, currentSupportsThinking);
      return;
    }
    setWelcomeReasoningMode(mode);
  }, [model, provider, normalizeForSelectedModel, tabState.activeSessionId, updateReasoning]);

  const updateModelWithReasoning = useCallback(
    async (id: string, nextModel: string, nextProvider: string) => {
      const nextReasoning = normalizeForSelectedModel(nextModel, nextProvider, reasoningMode);
      await updateModel(
        id,
        nextModel,
        nextProvider,
        nextReasoning.mode,
        nextReasoning.supportsThinking,
      );
    },
    [normalizeForSelectedModel, reasoningMode, updateModel],
  );

  const applySessionSelection = useCallback(async (id: string) => {
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

  const handleSelectById = useCallback(async (id: string) => {
    await applySessionSelection(id);
    onSessionChange?.(id);
  }, [applySessionSelection, onSessionChange]);

  useEffect(() => {
    if (navState.sessionId === tabState.activeSessionId) return;
    if (navState.sessionId === null) {
      void tabState.deselectTab();
    } else {
      void applySessionSelection(navState.sessionId);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- only react to requested session changes
  }, [navState.sessionId]);

  const filePreview = useAgentLocalControlledPreview({ navState, filePreviewState, onNavChange });
  const terminal = useAgentLocalControlledTerminal({ navState, terminalState, terminalCwd, onNavChange });

  useAgentLocalShortcuts({
    activeSessionId: tabState.activeSessionId,
    terminalOpen: terminal.isOpen,
    terminalTabsCount: terminal.tabs.length,
    terminalCwd,
    onAddTerminalTab: terminal.addTab,
    onToggleTerminal: terminal.togglePanel,
  });

  useAgentLocalTabPanelSync({ navState, filePreview: filePreviewState, terminal: terminalState });

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
    focusActiveSelector: "[data-nav-zone='list'] [data-nav-active='true']",
  });

  const handleDeleteProject = useCallback((projectId: string) => {
    const entries = terminalState.getGroupPtyEntries(projectId);
    for (const { id, token } of entries) {
      invoke("pty_kill", { id, token }).catch(() => {});
    }
    terminalState.removeGroup(projectId);
    void projectsHook.remove(projectId);
  }, [terminalState, projectsHook]);

  return {
    sessions, refresh, rename, remove, updateModel: updateModelWithReasoning,
    tabState, projectsHook, terminal, activeSession,
    model, provider, currentDefault, activeProject,
    filePreview, fileOperations, setFileOperations,
    reasoningMode, setReasoningMode, welcomeModel, setWelcomeModel,
    sessionActions, handleSelectById, handleDeleteProject,
  };
}

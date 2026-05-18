import { useState, useEffect, useCallback, useMemo, useLayoutEffect, memo } from "react";
import { useTranslation } from "react-i18next";
import type { PersonalityFile } from "@/types/personality";
import * as api from "@/services/personality";
import { useFsEvent } from "@/hooks/use-fs-event";
import { useArrowNavigation } from "@/hooks/use-arrow-navigation";
import { showToast } from "@/lib/toast-emitter";
import { PersonalityList } from "./personality-list";
import { MarkdownViewer } from "./markdown-viewer";
import type { TabSlots } from "@/components/agent-local/agent-local-tab-types";

interface PersonalityTabProps {
  activePath?: string | null;
  onPathChange?: (path: string | null) => void;
  listFocused?: boolean;
  reportContent: (slots: TabSlots) => void;
}

export const PersonalityTab = memo(function PersonalityTab({
  activePath,
  onPathChange,
  listFocused = true,
  reportContent,
}: PersonalityTabProps) {
  const { t } = useTranslation();
  const [files, setFiles] = useState<PersonalityFile[]>([]);
  const [selectedPath, setSelectedPathState] = useState<string | null>(null);

  useEffect(() => {
    if (activePath !== undefined) setSelectedPathState(activePath);
  }, [activePath]);
  const [content, setContent] = useState<string>("");
  const [fileName, setFileName] = useState<string>("");
  const [injectionState, setInjectionState] = useState<Record<string, boolean>>({});

  const loadFiles = useCallback(() => {
    api.listFiles().then(setFiles).catch(() => showToast(t("personality.failedToLoad")));
  }, [t]);

  const loadInjectionState = useCallback(() => {
    api.getInjectionState().then(setInjectionState).catch(() => {});
  }, []);

  useEffect(() => {
    loadFiles();
    loadInjectionState();
  }, [loadFiles, loadInjectionState]);

  useFsEvent("fs:personality-changed", loadFiles);

  const reloadContent = useCallback(() => {
    if (selectedPath) {
      api.readFile(selectedPath).then(setContent).catch(() => showToast(t("personality.failedToLoad")));
    }
  }, [selectedPath, t]);
  useFsEvent("fs:personality-changed", reloadContent);

  const setSelectedPath = useCallback((path: string | null) => {
    setSelectedPathState(path);
    onPathChange?.(path);
  // eslint-disable-next-line react-hooks/exhaustive-deps -- onPathChange identity is stable per caller
  }, []);

  const handleSelect = useCallback(async (path: string) => {
    setSelectedPath(path);
    try {
      const text = await api.readFile(path);
      setContent(text);
      setFileName(path.split(/[\\/]/).pop() ?? "");
    } catch {
      showToast(t("personality.failedToRead"));
      setContent(t("errors.readError"));
    }
  }, [setSelectedPath, t]);

  useEffect(() => {
    if (!selectedPath && files.length > 0) {
      void handleSelect(files[0].path);
    }
  }, [files, selectedPath, handleSelect]);

  const handleOpen = useCallback(() => {
    if (selectedPath) {
      api.openInEditor(selectedPath).catch(() => showToast(t("personality.failedToLoad")));
    }
  }, [selectedPath, t]);

  const handleToggleInjection = useCallback(async (enabled: boolean) => {
    if (fileName === "AGENTS.md") return;
    try {
      await api.setInjectionState(fileName, enabled);
      setInjectionState((prev) => ({ ...prev, [fileName]: enabled }));
    } catch {
      showToast(t("personality.failedToLoad"));
    }
  }, [fileName, t]);

  const filePaths = useMemo(() => files.map((f) => f.path), [files]);
  useArrowNavigation({
    items: filePaths,
    selectedId: selectedPath,
    onSelect: (path) => void handleSelect(path),
    enabled: listFocused,
    focusActiveSelector: "[data-nav-zone='list'] [data-nav-active='true']",
  });

  const list = (
    <PersonalityList
      files={files}
      selectedPath={selectedPath}
      injectionState={injectionState}
      selectedFileName={fileName}
      onSelect={(path) => void handleSelect(path)}
      onToggleInjection={(enabled) => void handleToggleInjection(enabled)}
    />
  );

  let detail: React.ReactNode;
  if (!selectedPath) {
    detail = (
      <div style={{ padding: "var(--space-lg)", color: "var(--ink-faint)" }}>
        {t("personality.selectFile")}
      </div>
    );
  } else {
    detail = (
      <MarkdownViewer
        content={content}
        fileName={fileName}
        onOpenEditor={handleOpen}
      />
    );
  }

  // eslint-disable-next-line react-hooks/exhaustive-deps -- reports the fresh slots from this render
  useLayoutEffect(() => { reportContent({ list, detail }); }, [
    reportContent, files, selectedPath, content, fileName, injectionState, listFocused,
  ]);

  return null;
});

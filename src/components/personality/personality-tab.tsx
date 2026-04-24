import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import type { PersonalityFile } from "@/types/personality";
import * as api from "@/services/personality";
import { useFsEvent } from "@/hooks/use-fs-event";
import { showToast } from "@/lib/toast-emitter";
import { PersonalityList } from "./personality-list";
import { MarkdownViewer } from "./markdown-viewer";

interface PersonalityTabProps {
  activePath?: string | null;
  onPathChange?: (path: string | null) => void;
}

export function PersonalityTab(props?: PersonalityTabProps): { list: React.ReactNode; detail: React.ReactNode } {
  const { t } = useTranslation();
  const [files, setFiles] = useState<PersonalityFile[]>([]);
  const [selectedPath, setSelectedPathState] = useState<string | null>(null);

  useEffect(() => {
    if (props?.activePath !== undefined) setSelectedPathState(props.activePath);
  }, [props?.activePath]);
  const [content, setContent] = useState<string>("");
  const [fileName, setFileName] = useState<string>("");
  const [injectionState, setInjectionState] = useState<Record<string, boolean>>({});

  const loadFiles = useCallback(() => {
    api.listFiles().then(setFiles).catch(() => showToast(t("personality.failedToLoad")));
  }, []);

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
  }, [selectedPath]);
  useFsEvent("fs:personality-changed", reloadContent);

  const setSelectedPath = (path: string | null) => {
    setSelectedPathState(path);
    props?.onPathChange?.(path);
  };

  const handleSelect = useCallback(async (path: string) => {
    setSelectedPath(path);
    try {
      const text = await api.readFile(path);
      setContent(text);
      setFileName(path.split(/[\\/]/).pop() ?? "");
    } catch {
      showToast(t("personality.failedToRead"));
      setContent("Failed to read file");
    }
  }, []);

  useEffect(() => {
    if (!selectedPath && files.length > 0) {
      handleSelect(files[0].path);
    }
  }, [files, selectedPath, handleSelect]);

  const handleOpen = useCallback(() => {
    if (selectedPath) {
      api.openInEditor(selectedPath).catch(() => showToast(t("personality.failedToLoad")));
    }
  }, [selectedPath]);

  const handleToggleInjection = useCallback(async (enabled: boolean) => {
    if (fileName === "AGENT.md") return;
    try {
      await api.setInjectionState(fileName, enabled);
      setInjectionState((prev) => ({ ...prev, [fileName]: enabled }));
    } catch {
      showToast(t("personality.failedToLoad"));
    }
  }, [fileName]);

  const list = (
    <PersonalityList
      files={files}
      selectedPath={selectedPath}
      injectionState={injectionState}
      selectedFileName={fileName}
      onSelect={handleSelect}
      onToggleInjection={handleToggleInjection}
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

  return { list, detail };
}

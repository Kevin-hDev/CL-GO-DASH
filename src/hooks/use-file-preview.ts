import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { fileNameFromPath, normalizeFileOperationPath } from "@/lib/file-preview-utils";
import type { AgentPlanRun } from "@/types/agent";
import type { FileOperation, FilePreviewActiveTab } from "@/types/file-preview";
import {
  clampFilePreviewWidthForContainer,
  readStoredFilePreviewTabs,
  writeStoredFilePreviewTabs,
} from "./file-preview-storage";
import {
  findOpenPreviewPanel,
  measurePreviewFullscreenWidth,
  measurePreviewLayout,
} from "./file-preview-layout";
import { beginPanelResize } from "./panel-resize";
import { useFilePreviewPanelState } from "./use-file-preview-panel-state";
import { usePreviewFallbackExistence } from "./use-preview-fallback-existence";
import { usePrunePreviewTabs } from "./use-prune-preview-tabs";
const MAX_TABS = 6;

export function useFilePreview(
  sessionId: string | null,
  operations: FileOperation[],
  baseDir?: string,
) {
  const {
    open,
    fullscreen,
    width,
    extraWidth,
    setOpen,
    setFullscreen,
    setWidth,
    setExtraWidth,
  } = useFilePreviewPanelState(sessionId);
  const [activeTab, setActiveTab] = useState<FilePreviewActiveTab>("summary");
  const [tabIds, setTabIds] = useState<string[]>(() => readStoredFilePreviewTabs(sessionId));
  const [fallbackOps, setFallbackOps] = useState<FileOperation[]>([]);
  const [resizing, setResizing] = useState(false);
  const [fullscreenWidth, setFullscreenWidth] = useState(() => (
    typeof window === "undefined" ? width : window.innerWidth
  ));
  const resizeRef = useRef<{
    startX: number;
    startWidth: number;
    container: Element | null;
    reservedWidth: number;
    chatMinWidth: number;
  } | null>(null);
  const stopResizeRef = useRef<(() => void) | null>(null);

  const allOperations = useMemo(() => [...operations, ...fallbackOps], [operations, fallbackOps]);
  const operationById = useMemo(() => new Map(allOperations.map((op) => [op.id, op])), [allOperations]);
  const tabs = tabIds.flatMap((id) => operationById.get(id) ?? []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- reset on session change is intentional
    setFallbackOps([]);
    setTabIds(readStoredFilePreviewTabs(sessionId));
    setActiveTab("summary");
  }, [sessionId]);

  usePrunePreviewTabs(operationById, setTabIds, setActiveTab);

  useEffect(() => {
    writeStoredFilePreviewTabs(sessionId, tabIds);
  }, [sessionId, tabIds]);

  const removeMissingFallbacks = useCallback((missingKeys: Set<string>) => {
    setFallbackOps((items) => items.filter((item) => (
      !missingKeys.has(normalizeFileOperationPath(item.path))
    )));
  }, []);
  usePreviewFallbackExistence(fallbackOps, baseDir, removeMissingFallbacks);

  const openOperation = useCallback((operation: FileOperation) => {
    setOpen(true);
    setActiveTab(operation.id);
    setTabIds((ids) => {
      const next = [operation.id, ...ids.filter((id) => id !== operation.id)];
      return next.slice(0, MAX_TABS);
    });
    return operation.id;
  }, [setOpen]);

  const openFullPath = useCallback((path: string) => {
    const fallback: FileOperation = {
      id: `read:${path}`,
      path,
      name: fileNameFromPath(path),
      type: "read",
      timestamp: new Date().toISOString(),
      additions: 0,
      deletions: 0,
    };
    setFallbackOps((items) => [fallback, ...items.filter((item) => item.id !== fallback.id)].slice(0, MAX_TABS));
    return openOperation(fallback);
  }, [openOperation]);

  const openPath = useCallback((path: string) => {
    const operation = [...operations].reverse().find((op) => op.path === path);
    if (operation) {
      return openOperation(operation);
    }
    return openFullPath(path);
  }, [operations, openOperation, openFullPath]);

  const openPlan = useCallback((plan: AgentPlanRun) => {
    const operation: FileOperation = {
      id: `plan:${plan.id}`,
      path: plan.path,
      name: plan.title,
      type: "read",
      kind: "plan",
      timestamp: plan.updated_at,
      additions: 0,
      deletions: 0,
    };
    setFallbackOps((items) => [operation, ...items.filter((item) => item.id !== operation.id)].slice(0, MAX_TABS));
    return openOperation(operation);
  }, [openOperation]);

  const closeTab = useCallback((id: string) => {
    setTabIds((ids) => ids.filter((tabId) => tabId !== id));
    setActiveTab((current) => current === id ? "summary" : current);
  }, []);

  const closePanel = useCallback(() => {
    setOpen(false);
    setFullscreen(false);
    setExtraWidth(0);
  }, [setOpen, setFullscreen, setExtraWidth]);

  const toggleOpen = useCallback(() => {
    if (open) setFullscreen(false);
    setOpen(!open);
    setActiveTab((current) => current || "summary");
  }, [open, setOpen, setFullscreen]);

  const startResize = useCallback((event: React.PointerEvent) => {
    const target = event.currentTarget as HTMLElement;
    const panel = target.closest(".fp-panel");
    const layout = measurePreviewLayout(panel, extraWidth);
    stopResizeRef.current?.();
    stopResizeRef.current = beginPanelResize(event, ".fp-panel");
    resizeRef.current = {
      startX: event.clientX,
      startWidth: width,
      container: layout.container,
      reservedWidth: layout.reservedWidth,
      chatMinWidth: layout.chatMinWidth,
    };
    setResizing(true);
  }, [width, extraWidth]);

  useEffect(() => {
    if (!open) return;
    const panel = findOpenPreviewPanel();
    const updateWidth = () => setFullscreenWidth((current) => {
      const next = measurePreviewFullscreenWidth(panel);
      return next === current ? current : next;
    });
    updateWidth();
    const layout = measurePreviewLayout(panel, 0);
    if (!layout.container || typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(updateWidth);
    observer.observe(layout.container);
    for (const child of layout.container.children) {
      if (child !== panel && !child.classList.contains("agent-detail-chat")) observer.observe(child);
    }
    return () => observer.disconnect();
  }, [open]);

  useEffect(() => {
    const onMove = (event: PointerEvent) => {
      if (!resizeRef.current) return;
      const delta = resizeRef.current.startX - event.clientX;
      const containerWidth = resizeRef.current.container?.getBoundingClientRect().width ?? window.innerWidth;
      setWidth(clampFilePreviewWidthForContainer(
        resizeRef.current.startWidth + delta,
        containerWidth,
        resizeRef.current.reservedWidth,
        resizeRef.current.chatMinWidth,
      ));
    };
    const stopResize = () => {
      resizeRef.current = null;
      stopResizeRef.current?.();
      stopResizeRef.current = null;
      setResizing(false);
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", stopResize);
    window.addEventListener("pointercancel", stopResize);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", stopResize);
      window.removeEventListener("pointercancel", stopResize);
      stopResizeRef.current?.();
      stopResizeRef.current = null;
    };
  }, [setWidth]);

  return {
    open,
    fullscreen,
    activeTab,
    tabs,
    width,
    extraWidth,
    fullscreenWidth,
    resizing,
    setOpen,
    setFullscreen,
    setExtraWidth,
    setActiveTab,
    toggleOpen,
    closePanel,
    openOperation,
    openPath,
    openFullPath,
    openPlan,
    closeTab,
    startResize,
  };
}

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { fileNameFromPath } from "@/lib/file-preview-utils";
import { IS_MAC } from "@/lib/platform";
import type { FileOperation, FilePreviewActiveTab } from "@/types/file-preview";
import {
  FILE_PREVIEW_DEFAULT_EXTRA_WIDTH,
  FILE_PREVIEW_MIN_WIDTH,
  readStoredFilePreviewPanel,
  readStoredFilePreviewTabs,
  writeStoredFilePreviewPanel,
  writeStoredFilePreviewTabs,
  type StoredFilePreviewPanel,
} from "./file-preview-storage";

const MAX_TABS = 6;

export function useFilePreview(sessionId: string | null, operations: FileOperation[]) {
  const [open, setOpen] = useState(() => readStoredFilePreviewPanel(sessionId).open);
  const [fullscreen, setFullscreen] = useState(() => readStoredFilePreviewPanel(sessionId).fullscreen);
  const [activeTab, setActiveTab] = useState<FilePreviewActiveTab>("summary");
  const [tabIds, setTabIds] = useState<string[]>(() => readStoredFilePreviewTabs(sessionId));
  const [fallbackOps, setFallbackOps] = useState<FileOperation[]>([]);
  const [width, setWidth] = useState(() => readStoredFilePreviewPanel(sessionId).width);
  const [extraWidth, setExtraWidth] = useState(FILE_PREVIEW_DEFAULT_EXTRA_WIDTH);
  const [resizing, setResizing] = useState(false);
  const resizeRef = useRef<{ startX: number; startWidth: number } | null>(null);
  const skipPanelPersistRef = useRef(false);

  const allOperations = useMemo(() => [...operations, ...fallbackOps], [operations, fallbackOps]);
  const operationById = useMemo(() => new Map(allOperations.map((op) => [op.id, op])), [allOperations]);
  const tabs = tabIds.flatMap((id) => operationById.get(id) ?? []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- reset on session change is intentional
    setFallbackOps([]);
    const valid = new Set(operations.map((op) => op.id));
    setTabIds(readStoredFilePreviewTabs(sessionId).filter((id) => valid.has(id)));
    setActiveTab("summary");
  }, [sessionId, operations]);

  useEffect(() => {
    const stored = readStoredFilePreviewPanel(sessionId);
    skipPanelPersistRef.current = true;
    // eslint-disable-next-line react-hooks/set-state-in-effect -- restore persisted panel state when switching sessions
    setOpen(stored.open);
    setFullscreen(stored.fullscreen);
    setWidth(stored.width);
    setExtraWidth(FILE_PREVIEW_DEFAULT_EXTRA_WIDTH);
  }, [sessionId]);

  useEffect(() => {
    writeStoredFilePreviewTabs(sessionId, tabIds);
  }, [sessionId, tabIds]);

  useEffect(() => {
    if (skipPanelPersistRef.current) {
      skipPanelPersistRef.current = false;
      return;
    }
    const state: StoredFilePreviewPanel = { open, fullscreen, width };
    writeStoredFilePreviewPanel(sessionId, state);
  }, [sessionId, open, fullscreen, width]);

  const openOperation = useCallback((operation: FileOperation) => {
    setOpen(true);
    setActiveTab(operation.id);
    setTabIds((ids) => {
      const next = [operation.id, ...ids.filter((id) => id !== operation.id)];
      return next.slice(0, MAX_TABS);
    });
  }, []);

  const openPath = useCallback((path: string) => {
    const operation = [...operations].reverse().find((op) => op.path === path);
    if (operation) {
      openOperation(operation);
      return;
    }
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
    openOperation(fallback);
  }, [operations, openOperation]);

  const closeTab = useCallback((id: string) => {
    setTabIds((ids) => ids.filter((tabId) => tabId !== id));
    setActiveTab((current) => current === id ? "summary" : current);
  }, []);

  const closePanel = useCallback(() => {
    setOpen(false);
    setFullscreen(false);
    setExtraWidth(FILE_PREVIEW_DEFAULT_EXTRA_WIDTH);
  }, []);

  const toggleOpen = useCallback(() => {
    setOpen((value) => {
      if (value) setFullscreen(false);
      return !value;
    });
    setActiveTab((current) => current || "summary");
  }, []);

  const startResize = useCallback((event: React.PointerEvent) => {
    event.preventDefault();
    resizeRef.current = { startX: event.clientX, startWidth: width };
    setResizing(true);
  }, [width]);

  useEffect(() => {
    const onMove = (event: PointerEvent) => {
      if (!resizeRef.current) return;
      const delta = resizeRef.current.startX - event.clientX;
      const maxWidth = Math.max(FILE_PREVIEW_MIN_WIDTH, window.innerWidth - 120);
      setWidth(Math.min(maxWidth, Math.max(FILE_PREVIEW_MIN_WIDTH, resizeRef.current.startWidth + delta)));
    };
    const onUp = () => {
      resizeRef.current = null;
      setResizing(false);
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
  }, []);

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      const mod = IS_MAC ? event.metaKey : event.ctrlKey;
      if (!mod || !event.altKey || event.code !== "KeyB") return;
      event.preventDefault();
      toggleOpen();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [toggleOpen]);

  return {
    open,
    fullscreen,
    activeTab,
    tabs,
    width,
    extraWidth,
    resizing,
    setOpen,
    setFullscreen,
    setExtraWidth,
    setActiveTab,
    toggleOpen,
    closePanel,
    openOperation,
    openPath,
    closeTab,
    startResize,
  };
}

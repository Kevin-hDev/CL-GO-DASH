import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { fileNameFromPath } from "@/lib/file-preview-utils";
import { IS_MAC } from "@/lib/platform";
import type { FileOperation, FilePreviewActiveTab } from "@/types/file-preview";

const MAX_TABS = 6;
const MIN_WIDTH = 360;
const DEFAULT_WIDTH = 520;
const MAX_STORED_TABS = 6;

function storageKey(sessionId: string | null): string {
  return `clgo-file-preview-tabs:${sessionId ?? "none"}`;
}

function readStoredTabs(sessionId: string | null): string[] {
  try {
    const raw = localStorage.getItem(storageKey(sessionId));
    const parsed = JSON.parse(raw ?? "[]");
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((id) => typeof id === "string").slice(0, MAX_STORED_TABS);
  } catch {
    return [];
  }
}

export function useFilePreview(sessionId: string | null, operations: FileOperation[]) {
  const [open, setOpen] = useState(false);
  const [fullscreen, setFullscreen] = useState(false);
  const [activeTab, setActiveTab] = useState<FilePreviewActiveTab>("summary");
  const [tabIds, setTabIds] = useState<string[]>(() => readStoredTabs(sessionId));
  const [fallbackOps, setFallbackOps] = useState<FileOperation[]>([]);
  const [width, setWidth] = useState(DEFAULT_WIDTH);
  const resizeRef = useRef<{ startX: number; startWidth: number } | null>(null);

  const allOperations = useMemo(() => [...operations, ...fallbackOps], [operations, fallbackOps]);
  const operationById = useMemo(() => new Map(allOperations.map((op) => [op.id, op])), [allOperations]);
  const tabs = tabIds.flatMap((id) => operationById.get(id) ?? []);

  useEffect(() => {
    setFallbackOps([]);
    const valid = new Set(operations.map((op) => op.id));
    setTabIds(readStoredTabs(sessionId).filter((id) => valid.has(id)));
    setActiveTab("summary");
  }, [sessionId]);

  useEffect(() => {
    localStorage.setItem(storageKey(sessionId), JSON.stringify(tabIds.slice(0, MAX_STORED_TABS)));
  }, [sessionId, tabIds]);

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

  const toggleOpen = useCallback(() => {
    setOpen((value) => !value);
    setActiveTab((current) => current || "summary");
  }, []);

  const startResize = useCallback((event: React.PointerEvent) => {
    event.preventDefault();
    resizeRef.current = { startX: event.clientX, startWidth: width };
  }, [width]);

  useEffect(() => {
    const onMove = (event: PointerEvent) => {
      if (!resizeRef.current) return;
      const delta = resizeRef.current.startX - event.clientX;
      const maxWidth = Math.max(MIN_WIDTH, window.innerWidth - 120);
      setWidth(Math.min(maxWidth, Math.max(MIN_WIDTH, resizeRef.current.startWidth + delta)));
    };
    const onUp = () => {
      resizeRef.current = null;
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
    setOpen,
    setFullscreen,
    setActiveTab,
    toggleOpen,
    openOperation,
    openPath,
    closeTab,
    startResize,
  };
}

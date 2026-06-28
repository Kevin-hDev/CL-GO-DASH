import { CHAT_MIN_WIDTH } from "./file-preview-storage";
import { siblingPanelWidth } from "./file-preview-layout";

export const FILE_TREE_DEFAULT_WIDTH = 240;
export const FILE_TREE_MIN_WIDTH = 160;
export const FILE_TREE_MAX_WIDTH = 500;

export interface FileTreeLayout {
  container: Element | null;
  containerWidth: number;
  reservedWidth: number;
}

export function findOpenFileTreePanel(): Element | null {
  return document.querySelector(".ft-panel.open");
}

export function clampFileTreeWidthForContainer(
  value: unknown,
  containerWidth: number,
  reservedWidth = 0,
): number {
  const available = containerWidth - CHAT_MIN_WIDTH - Math.max(0, reservedWidth);
  const maxWidth = Math.min(FILE_TREE_MAX_WIDTH, Math.max(0, available));
  const minWidth = Math.min(FILE_TREE_MIN_WIDTH, maxWidth);
  const width = typeof value === "number" && Number.isFinite(value) ? value : FILE_TREE_DEFAULT_WIDTH;
  return Math.min(maxWidth, Math.max(minWidth, width));
}

export function clampFileTreeStoredWidth(value: unknown): number {
  const width = typeof value === "number" && Number.isFinite(value) ? value : FILE_TREE_DEFAULT_WIDTH;
  return Math.min(FILE_TREE_MAX_WIDTH, Math.max(FILE_TREE_MIN_WIDTH, width));
}

export function measureFileTreeLayout(panel: Element | null): FileTreeLayout {
  const container = panel?.closest(".agent-detail-with-preview") ?? null;
  return {
    container,
    containerWidth: container?.getBoundingClientRect().width ?? window.innerWidth,
    reservedWidth: siblingPanelWidth(panel, container),
  };
}

export function clampTreeWidthForLayout(width: number, panel: Element | null): number {
  const layout = measureFileTreeLayout(panel);
  return clampFileTreeWidthForContainer(width, layout.containerWidth, layout.reservedWidth);
}

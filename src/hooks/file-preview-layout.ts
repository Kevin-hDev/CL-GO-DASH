import { clampFilePreviewWidthForContainer } from "./file-preview-storage";

export interface FilePreviewLayout {
  container: Element | null;
  containerWidth: number;
  reservedWidth: number;
}

export function findOpenPreviewPanel(): Element | null {
  return document.querySelector(".fp-panel.open");
}

export function siblingPanelWidth(panel: Element | null, container: Element | null): number {
  if (!container || !panel) return 0;
  return [...container.children].reduce((total, child) => {
    if (child === panel || child.classList.contains("agent-detail-chat")) return total;
    return total + child.getBoundingClientRect().width;
  }, 0);
}

export function measurePreviewLayout(panel: Element | null, extraWidth: number): FilePreviewLayout {
  const container = panel?.closest(".agent-detail-with-preview") ?? null;
  return {
    container,
    containerWidth: container?.getBoundingClientRect().width ?? window.innerWidth,
    reservedWidth: extraWidth + siblingPanelWidth(panel, container),
  };
}

export function clampPreviewWidthForLayout(width: number, panel: Element | null, extraWidth: number): number {
  const layout = measurePreviewLayout(panel, extraWidth);
  return clampFilePreviewWidthForContainer(width, layout.containerWidth, layout.reservedWidth);
}

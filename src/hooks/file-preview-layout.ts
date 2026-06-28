import { CHAT_MIN_WIDTH, clampFilePreviewWidthForContainer } from "./file-preview-storage";

export interface FilePreviewLayout {
  container: Element | null;
  containerWidth: number;
  reservedWidth: number;
  chatMinWidth: number;
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

export function chatMinWidthForContainer(container: Element | null): number {
  if (!(container instanceof HTMLElement)) return CHAT_MIN_WIDTH;
  const value = Number.parseFloat(getComputedStyle(container).getPropertyValue("--agent-chat-min-width"));
  return Number.isFinite(value) ? Math.max(0, value) : CHAT_MIN_WIDTH;
}

export function measurePreviewLayout(panel: Element | null, extraWidth: number): FilePreviewLayout {
  const container = panel?.closest(".agent-detail-with-preview") ?? null;
  return {
    container,
    containerWidth: container?.getBoundingClientRect().width ?? window.innerWidth,
    reservedWidth: extraWidth + siblingPanelWidth(panel, container),
    chatMinWidth: chatMinWidthForContainer(container),
  };
}

export function clampPreviewWidthForLayout(width: number, panel: Element | null, extraWidth: number): number {
  const layout = measurePreviewLayout(panel, extraWidth);
  return clampFilePreviewWidthForContainer(width, layout.containerWidth, layout.reservedWidth, layout.chatMinWidth);
}

export function measurePreviewFullscreenWidth(panel: Element | null): number {
  const layout = measurePreviewLayout(panel, 0);
  return Math.max(0, layout.containerWidth - layout.reservedWidth);
}

import i18n from "@/i18n";
import type { StreamEvent } from "@/types/agent";

const MAX_KEYS = 128;
const MAX_MESSAGE_CHARS = 300;
const TOASTED_WEB_TOOLS = new Set(["web_search"]);
const shownKeys: string[] = [];
const shown = new Set<string>();

export function webToolErrorToastMessage(sessionId: string, event: StreamEvent): string | null {
  if (event.event !== "toolResult" || !event.data.isError) return null;
  if (!TOASTED_WEB_TOOLS.has(event.data.name)) return null;

  const detail = sanitizeWebToolError(event.data.content);

  const key = `${sessionId}:${event.data.name}:${event.data.toolCallIndex}`;
  if (shown.has(key)) return null;
  remember(key);

  return i18n.t("errors.webToolFailed", { message: detail || i18n.t("errors.toolFailed") });
}

function remember(key: string) {
  shown.add(key);
  shownKeys.push(key);
  while (shownKeys.length > MAX_KEYS) {
    const old = shownKeys.shift();
    if (old) shown.delete(old);
  }
}

export function sanitizeWebToolError(input: string): string {
  const firstLine = input.split(/\r?\n/).find(Boolean) ?? "";
  return truncate(firstLine)
    .replace(/(bearer\s+)[a-z0-9._~+/=-]{8,}/gi, "$1[redacted]")
    .replace(/(api[_-]?key|token|secret|password)\s*[:=]\s*[^;\s]+/gi, "$1=[redacted]")
    .replace(/\/Users\/[^\s;]+/g, "[path]")
    .replace(/[A-Z]:\\[^\s;]+/g, "[path]");
}

function truncate(input: string): string {
  const chars = [...input];
  if (chars.length <= MAX_MESSAGE_CHARS) return input;
  return `${chars.slice(0, MAX_MESSAGE_CHARS).join("")}...`;
}

export function resetWebToolToastDedupeForTest() {
  shown.clear();
  shownKeys.length = 0;
}

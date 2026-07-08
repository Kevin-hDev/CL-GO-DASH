import type { AgentMessage, SubagentInfo, ToolActivityRecord } from "@/types/agent";

const MAX_MESSAGE_SUBAGENTS = 64;
const MAX_KNOWN_SUBAGENTS = 512;
const MAX_PROMPT_PREVIEW = 120;
const SESSION_ID_RE = /^[A-Za-z0-9_-]{1,128}$/;
const SUBAGENT_ID_RE = /<subagent\b[^>]*\bid="([^"]{1,128})"/;

export function collectMessageSubagents(
  message: AgentMessage,
  knownSubagents: SubagentInfo[] = [],
): SubagentInfo[] {
  const known = knownSubagentsById(knownSubagents);
  const collected: SubagentInfo[] = [];
  const seen = new Set<string>();

  for (const tool of messageSubagentTools(message)) {
    if (collected.length >= MAX_MESSAGE_SUBAGENTS) break;
    const sessionId = extractSubagentId(tool);
    if (!sessionId || seen.has(sessionId)) continue;
    seen.add(sessionId);
    collected.push(mergeSubagentInfo(fallbackSubagentInfo(sessionId, tool), known.get(sessionId)));
  }

  return collected;
}

function messageSubagentTools(message: AgentMessage): ToolActivityRecord[] {
  if (message.segments?.length) {
    return message.segments
      .filter((segment) => segment.phase !== "final")
      .flatMap((segment) => segment.tools)
      .filter((tool) => tool.name === "delegate_task");
  }
  return (message.tool_activities ?? []).filter((tool) => tool.name === "delegate_task");
}

function knownSubagentsById(subagents: SubagentInfo[]): Map<string, SubagentInfo> {
  const byId = new Map<string, SubagentInfo>();
  for (const subagent of subagents.slice(0, MAX_KNOWN_SUBAGENTS)) {
    if (!isSessionId(subagent.sessionId)) continue;
    byId.set(subagent.sessionId, subagent);
  }
  return byId;
}

function extractSubagentId(tool: ToolActivityRecord): string | null {
  const fromResult = typeof tool.result === "string"
    ? SUBAGENT_ID_RE.exec(tool.result)?.[1]
    : undefined;
  const candidate = fromResult || stringArg(tool.args, "subagent_id");
  return candidate && isSessionId(candidate) ? candidate : null;
}

function fallbackSubagentInfo(sessionId: string, tool: ToolActivityRecord): SubagentInfo {
  const type = stringArg(tool.args, "subagent_type") === "coder" ? "coder" : "explorer";
  return {
    sessionId,
    name: stringArg(tool.args, "display_name") || stringArg(tool.args, "name") || "agent",
    type,
    status: tool.is_error ? "failed" : "completed",
    promptPreview: limitChars(stringArg(tool.args, "prompt"), MAX_PROMPT_PREVIEW),
    description: stringArg(tool.args, "description") || undefined,
  };
}

function mergeSubagentInfo(fallback: SubagentInfo, known?: SubagentInfo): SubagentInfo {
  if (!known) return fallback;
  return {
    ...fallback,
    ...known,
    promptPreview: known.promptPreview || fallback.promptPreview,
    description: known.description || fallback.description,
  };
}

function stringArg(args: Record<string, unknown> | undefined, key: string): string {
  const value = args?.[key];
  return typeof value === "string" ? value.trim() : "";
}

function isSessionId(value: string): boolean {
  return SESSION_ID_RE.test(value);
}

function limitChars(value: string, maxChars: number): string {
  return Array.from(value).slice(0, maxChars).join("");
}

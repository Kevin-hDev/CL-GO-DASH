import { buildSegmentedMessage } from "./agent-chat-utils";
import type { StreamSegment, ToolActivity } from "./agent-chat-utils";
import type { AgentMessage, StreamEvent } from "@/types/agent";

const MAX_PENDING_PERMISSIONS = 32;

export interface ChatState {
  messages: AgentMessage[];
  completedSegments: StreamSegment[];
  currentContent: string;
  currentThinking: string;
  currentTools: ToolActivity[];
  isStreaming: boolean;
  tps: number;
  tokenCount: number;
  lastRequestTokens: number;
}

export interface PermissionRequestState {
  id: string;
  toolName: string;
  arguments: Record<string, unknown>;
}

export interface ManagedStreamState extends ChatState {
  pendingPermissions: PermissionRequestState[];
  completed: boolean;
  persisted: boolean;
  updatedAt: number;
  error?: string;
}

export const EMPTY_CHAT_STATE: ChatState = {
  messages: [], completedSegments: [], currentContent: "",
  currentThinking: "", currentTools: [], isStreaming: false,
  tps: 0, tokenCount: 0, lastRequestTokens: 0,
};

export function createManagedStreamState(
  messages: AgentMessage[],
  tokenCount: number,
): ManagedStreamState {
  return {
    ...EMPTY_CHAT_STATE, messages, tokenCount, isStreaming: true,
    pendingPermissions: [], completed: false, persisted: false,
    updatedAt: Date.now(),
  };
}

export function toChatState(state: ManagedStreamState): ChatState {
  return {
    messages: state.messages, completedSegments: state.completedSegments,
    currentContent: state.currentContent, currentThinking: state.currentThinking,
    currentTools: state.currentTools, isStreaming: state.isStreaming,
    tps: state.tps, tokenCount: state.tokenCount,
    lastRequestTokens: state.lastRequestTokens,
  };
}

export interface StreamApplyResult {
  state: ManagedStreamState;
  assistantMessage?: AgentMessage;
  assistantTokens?: number;
}

export function applyStreamEvent(
  state: ManagedStreamState,
  event: StreamEvent,
): StreamApplyResult {
  const next = { ...state, updatedAt: Date.now() };
  switch (event.event) {
    case "token":
      next.currentContent += event.data.content;
      next.tps = event.data.tps;
      break;
    case "thinking":
      next.currentThinking += event.data.content;
      break;
    case "toolCall":
      next.currentTools = [...next.currentTools, {
        name: event.data.name, args: event.data.arguments,
      }];
      break;
    case "toolResult":
      next.currentTools = applyToolResult(
        next.currentTools,
        event.data.name,
        event.data.content,
        event.data.isError,
      );
      next.pendingPermissions = [];
      break;
    case "turnEnd":
      next.completedSegments = appendCurrentSegment(next);
      next.currentContent = "";
      next.currentThinking = "";
      next.currentTools = [];
      break;
    case "permissionRequest":
      next.pendingPermissions = addPermission(next.pendingPermissions, { id: event.data.id,
        toolName: event.data.toolName, arguments: event.data.arguments });
      break;
    case "done":
      return finishStream(next, event);
    case "error":
      next.isStreaming = false;
      next.completed = true;
      next.error = "Le flux s'est interrompu.";
      break;
  }
  return { state: next };
}

function applyToolResult(
  tools: ToolActivity[],
  name: string,
  content: string,
  isError: boolean,
): ToolActivity[] {
  const next = [...tools];
  for (let i = next.length - 1; i >= 0; i--) {
    if (next[i].name === name && !next[i].result) {
      next[i] = { ...next[i], result: content, isError };
      break;
    }
  }
  return next;
}

function appendCurrentSegment(state: ChatState): StreamSegment[] {
  return [
    ...state.completedSegments,
    {
      thinking: state.currentThinking,
      tools: state.currentTools,
      content: state.currentContent,
    },
  ];
}

function addPermission(
  requests: PermissionRequestState[],
  request: PermissionRequestState,
): PermissionRequestState[] {
  const filtered = requests.filter((item) => item.id !== request.id);
  return [...filtered, request].slice(-MAX_PENDING_PERMISSIONS);
}

function finishStream(state: ManagedStreamState, event: Extract<StreamEvent, { event: "done" }>) {
  const all = state.currentContent || state.currentThinking || state.currentTools.length > 0
    ? appendCurrentSegment(state)
    : state.completedSegments;
  const tokens = (event.data.evalCount || 0) + (event.data.promptTokens || 0);
  const next = {
    ...state,
    completedSegments: [],
    currentContent: "",
    currentThinking: "",
    currentTools: [],
    isStreaming: false,
    tps: event.data.finalTps,
    tokenCount: state.tokenCount + tokens,
    lastRequestTokens: tokens,
    pendingPermissions: [],
    completed: true,
  };
  if (all.length === 0) return { state: next };
  const built = buildSegmentedMessage(all);
  const assistantMessage: AgentMessage = {
    id: crypto.randomUUID(),
    role: "assistant",
    content: built.content,
    thinking: built.thinking,
    tool_activities: built.toolRecords,
    segments: built.segments,
    files: [],
    timestamp: new Date().toISOString(),
    tokens,
  };
  return {
    state: { ...next, messages: [...next.messages, assistantMessage] },
    assistantMessage,
    assistantTokens: tokens,
  };
}

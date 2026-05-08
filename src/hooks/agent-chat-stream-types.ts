import type { StreamSegment, ToolActivity } from "./agent-chat-utils";
import type { AgentMessage } from "@/types/agent";

export const MAX_PENDING_PERMISSIONS = 32;
export const MAX_MESSAGES_PER_SESSION = 2000;

export const KNOWN_ERROR_KEYS: Record<string, string> = {
  ollama_connection_lost: "errors.ollamaConnectionLost",
  model_not_found: "errors.modelNotFound",
  context_length_exceeded: "errors.contextLengthExceeded",
  rate_limit: "errors.rateLimited",
  auth_failed: "errors.authFailed",
};

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
  liveTokenCount: number;
  streamStartedAt: number | null;
  segmentStartedAt: number | null;
  totalElapsedMs: number;
  error?: string;
  isConnectionError?: boolean;
}

export interface PermissionRequestState {
  id: string; toolName: string; arguments: Record<string, unknown>;
}

export interface ManagedStreamState extends ChatState {
  pendingPermissions: PermissionRequestState[];
  completed: boolean; persisted: boolean; updatedAt: number; error?: string; isConnectionError?: boolean;
}

export const EMPTY_CHAT_STATE: ChatState = {
  messages: [], completedSegments: [], currentContent: "",
  currentThinking: "", currentTools: [], isStreaming: false,
  tps: 0, tokenCount: 0, lastRequestTokens: 0,
  liveTokenCount: 0, streamStartedAt: null, segmentStartedAt: null,
  totalElapsedMs: 0,
};

export interface StreamApplyResult {
  state: ManagedStreamState;
  assistantMessage?: AgentMessage;
  assistantTokens?: number;
}

export function createManagedStreamState(
  messages: AgentMessage[],
  tokenCount: number,
): ManagedStreamState {
  const now = Date.now();
  return {
    ...EMPTY_CHAT_STATE, messages, tokenCount, isStreaming: true,
    streamStartedAt: now, segmentStartedAt: now,
    pendingPermissions: [], completed: false, persisted: false,
    updatedAt: now,
  };
}

export function toChatState(state: ManagedStreamState): ChatState {
  return {
    messages: state.messages, completedSegments: state.completedSegments,
    currentContent: state.currentContent, currentThinking: state.currentThinking,
    currentTools: state.currentTools, isStreaming: state.isStreaming,
    tps: state.tps, tokenCount: state.tokenCount,
    lastRequestTokens: state.lastRequestTokens,
    liveTokenCount: state.liveTokenCount,
    streamStartedAt: state.streamStartedAt,
    segmentStartedAt: state.segmentStartedAt,
    totalElapsedMs: state.totalElapsedMs,
    error: state.error,
    isConnectionError: state.isConnectionError,
  };
}

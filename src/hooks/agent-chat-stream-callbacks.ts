import { buildSegmentedMessage } from "./agent-chat-utils";
import type { StreamSegment } from "./agent-chat-utils";
import type { AgentMessage, StreamEvent, TokenPhase } from "@/types/agent";
import i18n from "@/i18n";
import { isHiddenAgentTool } from "@/lib/hidden-agent-tools";
import {
  MAX_PENDING_PERMISSIONS, MAX_MESSAGES_PER_SESSION, KNOWN_ERROR_KEYS,
  type ChatState, type ManagedStreamState, type StreamApplyResult,
  type PermissionRequestState,
} from "./agent-chat-stream-types";
import { estimateAgentMessagesTokens } from "./agent-token-estimate";
import { markUnconfirmedContentAsWork } from "./agent-chat-stream-partial";
import { lastPendingToolItem, thinkingItem, toolItem } from "./active-stream-item";
import { applyToolResult } from "./agent-chat-tool-results";

export type { ChatState, ManagedStreamState, PermissionRequestState, StreamApplyResult };
export { EMPTY_CHAT_STATE, createManagedStreamState, toChatState } from "./agent-chat-stream-types";

export function applyStreamEvent(
  state: ManagedStreamState,
  event: StreamEvent,
): StreamApplyResult {
  const now = Date.now();
  const next = { ...state, updatedAt: now };
  const ensureTimers = () => {
    if (!next.streamStartedAt) next.streamStartedAt = now;
    if (!next.segmentStartedAt) next.segmentStartedAt = now;
  };
  switch (event.event) {
    case "token":
      ensureTimers();
      next.retryIndicator = null;
      next.activeStreamItem = null;
      if (event.data.phase) prepareContentPhase(next, event.data.phase);
      next.currentContent += event.data.content;
      next.tps = event.data.tps;
      next.liveTokenCount = event.data.tokenCount || next.liveTokenCount + 1;
      break;
    case "contentPhase":
      ensureTimers();
      prepareContentPhase(next, event.data.phase);
      break;
    case "thinking":
      ensureTimers();
      next.currentThinking += event.data.content;
      next.activeStreamItem = thinkingItem();
      next.liveTokenCount += 1;
      break;
    case "toolCall":
      ensureTimers();
      if (isHiddenAgentTool(event.data.name)) break;
      next.activeStreamItem = toolItem(next.currentTools.length);
      next.currentTools = [...next.currentTools, {
        name: event.data.name, args: event.data.arguments,
      }];
      break;
    case "toolResult":
      if (isHiddenAgentTool(event.data.name)) {
        next.pendingPermissions = [];
        if (event.data.name === "ask_user_choice" || event.data.name === "planmode") {
          next.interactiveChoice = undefined;
        }
        break;
      }
      next.currentTools = applyToolResult(
        next.currentTools,
        event.data.toolCallIndex ?? -1,
        event.data.content,
        event.data.isError,
        event.data.resolvedPath,
        event.data.affectedPaths,
      );
      next.activeStreamItem = lastPendingToolItem(next.currentTools);
      next.pendingPermissions = [];
      break;
    case "turnEnd":
      next.retryIndicator = null;
      next.completedSegments = appendCurrentSegment(next);
      next.currentContent = "";
      next.currentContentPhase = undefined;
      next.currentThinking = "";
      next.currentTools = [];
      next.activeStreamItem = null;
      next.segmentStartedAt = null;
      break;
    case "permissionRequest":
      next.pendingPermissions = addPermission(next.pendingPermissions, { id: event.data.id,
        toolName: event.data.toolName, arguments: event.data.arguments });
      break;
    case "sessionSnapshot":
      break;
    case "subagentSpawned":
    case "subagentCompleted":
    case "todoUpdated":
      break;
    case "planPreviewUpdated":
      next.planPreview = event.data.plan;
      break;
    case "planModeUpdated":
      next.planModeEnabled = event.data.enabled;
      if (!event.data.enabled) next.planPreview = null;
      break;
    case "interactiveChoiceRequest":
      next.interactiveChoice = event.data;
      break;
    case "retryIndicator":
      next.retryIndicator = event.data;
      break;
    case "done":
      next.retryIndicator = null;
      return finishStream(next, event);
    case "error": {
      const rawMsg = event.data.message || "";
      const errorKey = KNOWN_ERROR_KEYS[rawMsg];
      next.error = errorKey ? i18n.t(errorKey) : i18n.t("errors.streamInterrupted");
      next.isConnectionError = (event.data as Record<string, unknown>).isConnection === true;
      next.diagnosticSummary = event.data.diagnostic?.safeSummary;
      const partial = finalizeStream(next, null, 0, null);
      partial.state.error = next.error;
      partial.state.isConnectionError = next.isConnectionError;
      partial.state.diagnosticSummary = next.diagnosticSummary;
      partial.state.retryIndicator = null;
      return partial;
    }
    case "notice":
      break;
  }
  return { state: next };
}

function appendCurrentSegment(state: ChatState): StreamSegment[] {
  return [...state.completedSegments, {
    thinking: state.currentThinking, tools: state.currentTools, content: state.currentContent,
    phase: state.currentContentPhase,
  }];
}

function prepareContentPhase(state: ManagedStreamState, phase: TokenPhase) {
  if (!state.currentContentPhase || state.currentContentPhase === phase) {
    state.currentContentPhase = phase;
    return;
  }
  if (state.currentContent || state.currentThinking || state.currentTools.length > 0) {
    state.completedSegments = appendCurrentSegment(state);
    state.currentContent = "";
    state.currentThinking = "";
    state.currentTools = [];
    state.activeStreamItem = null;
    state.segmentStartedAt = Date.now();
  }
  state.currentContentPhase = phase;
}

function addPermission(
  requests: PermissionRequestState[], request: PermissionRequestState,
): PermissionRequestState[] {
  return [...requests.filter((r) => r.id !== request.id), request].slice(-MAX_PENDING_PERMISSIONS);
}

export function finishPartialStream(state: ManagedStreamState): StreamApplyResult {
  return finalizeStream(markUnconfirmedContentAsWork(state), null, state.tps, null);
}

function finishStream(state: ManagedStreamState, event: Extract<StreamEvent, { event: "done" }>) {
  return finalizeStream(
    state,
    event.data.evalCount,
    event.data.finalTps,
    event.data.contextTokens,
  );
}

function finalizeStream(
  state: ManagedStreamState, outputTokens: number | null, tps: number, contextTokens: number | null,
): StreamApplyResult {
  const all = state.currentContent || state.currentThinking || state.currentTools.length > 0
    ? appendCurrentSegment(state) : state.completedSegments;
  const totalMs = state.streamStartedAt ? Date.now() - state.streamStartedAt : 0;
  const built = all.length > 0 ? buildSegmentedMessage(all) : null;
  const assistantMessage: AgentMessage | undefined = built ? {
    id: crypto.randomUUID(), role: "assistant", content: built.content,
    thinking: built.thinking, tool_activities: built.toolRecords,
    segments: built.segments, files: [], timestamp: new Date().toISOString(),
    tokens: 0, work_duration_ms: totalMs > 0 ? totalMs : undefined,
  } : undefined;
  if (assistantMessage) {
    const messageTokens = estimateAgentMessagesTokens([assistantMessage]);
    assistantMessage.tokens = outputTokens ?? messageTokens;
  }
  const allMessages = assistantMessage ? [...state.messages, assistantMessage] : [...state.messages];
  if (allMessages.length > MAX_MESSAGES_PER_SESSION) {
    allMessages.splice(0, allMessages.length - MAX_MESSAGES_PER_SESSION);
  }
  const visibleSessionTokens = estimateAgentMessagesTokens(allMessages);
  const hasRealContextTokens = contextTokens !== null;
  const resolvedSessionTokenCount = hasRealContextTokens ? contextTokens : visibleSessionTokens;
  const next: ManagedStreamState = {
    ...state, completedSegments: [], currentContent: "", currentThinking: "",
    currentContentPhase: undefined, currentTools: [], activeStreamItem: null,
    isStreaming: false, tps,
    sessionTokenCount: resolvedSessionTokenCount,
    sessionTokenCountEstimated: !hasRealContextTokens,
    lastRequestTokens: assistantMessage?.tokens ?? outputTokens ?? 0, liveTokenCount: 0,
    streamStartedAt: null, segmentStartedAt: null, totalElapsedMs: totalMs,
    pendingPermissions: [], interactiveChoice: undefined,
    completed: true, updatedAt: Date.now(),
  };
  if (all.length === 0) return { state: next };
  if (!assistantMessage) return { state: next };
  return {
    state: { ...next, messages: allMessages },
    assistantMessage, assistantTokens: assistantMessage.tokens ?? outputTokens ?? 0,
  };
}

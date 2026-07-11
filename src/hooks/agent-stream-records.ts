import { invoke } from "@tauri-apps/api/core";
import {
  createManagedStreamState,
  toChatState,
  type ChatState,
  type PermissionRequestState,
} from "./agent-chat-stream-callbacks";
import { enforceSessionLimit, type StreamRecord } from "./agent-stream-cleanup";
import { flushFrameNotify } from "./agent-stream-notify";
import type { AgentMessage } from "@/types/agent";

export interface StreamSnapshot extends ChatState {
  pendingPermissions: PermissionRequestState[];
  completed: boolean;
  error?: string;
  isConnectionError?: boolean;
  diagnosticSummary?: string;
}

export const records = new Map<string, StreamRecord>();

export function getRecord(sessionId: string): StreamRecord | undefined {
  return records.get(sessionId);
}

export function getOrCreateRecord(sessionId: string): StreamRecord {
  let record = records.get(sessionId);
  if (record) return record;
  record = {
    state: { ...createManagedStreamState([], 0), isStreaming: false },
    history: [],
    subscribers: new Map(),
    nextSubscriberId: 1,
    cleanupTimer: null,
    notifyHandle: null,
    started: false,
    backendOwnsPersistence: false,
    isSubagentBackendStream: false,
    activeGeneration: null,
    cancelledGenerations: [],
    cancelledWithoutGeneration: false,
  };
  records.set(sessionId, record);
  enforceSessionLimit(records);
  return record;
}

export function touchSession(sessionId: string, record: StreamRecord) {
  records.delete(sessionId);
  records.set(sessionId, record);
  enforceSessionLimit(records);
}

export function snapshot(state: StreamRecord["state"]): StreamSnapshot {
  return {
    ...toChatState(state), pendingPermissions: [...state.pendingPermissions],
    completed: state.completed, error: state.error,
    isConnectionError: state.isConnectionError,
    diagnosticSummary: state.diagnosticSummary,
  };
}

export function persistAssistant(
  sessionId: string,
  record: StreamRecord,
  message: AgentMessage,
  tokens: number,
  notify: (record: StreamRecord) => void,
) {
  record.state = { ...record.state, persisted: true };
  invoke("add_messages_to_session", {
    id: sessionId,
    messages: [message],
    tokens,
  }).catch(() => {
    console.warn("persist failed for session", sessionId.slice(0, 8));
    record.state = { ...record.state, persisted: false };
    flushFrameNotify(record, notify);
  });
}

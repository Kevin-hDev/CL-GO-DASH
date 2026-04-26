import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import i18n from "@/i18n";
import {
  applyStreamEvent, createManagedStreamState, toChatState,
  finishPartialStream,
  type ChatState, type ManagedStreamState, type PermissionRequestState,
} from "./agent-chat-stream-callbacks";
import type { AgentMessage, StreamEvent } from "@/types/agent";

const EVENT_NAME = "agent-stream-event";
const MAX_SESSIONS = 64;
const MAX_EVENTS_PER_SESSION = 4096;
const MAX_SUBSCRIBERS_PER_SESSION = 32;
const CLEANUP_DELAY_MS = 5 * 60 * 1000;

interface StreamEnvelope { sessionId: string; event: StreamEvent }

export interface StreamSnapshot extends ChatState {
  pendingPermissions: PermissionRequestState[];
  completed: boolean;
  error?: string;
  isConnectionError?: boolean;
}

type Subscriber = (snapshot: StreamSnapshot) => void;

interface StreamRecord {
  state: ManagedStreamState;
  history: StreamEvent[];
  subscribers: Map<number, Subscriber>;
  nextSubscriberId: number;
  cleanupTimer: ReturnType<typeof setTimeout> | null;
  started: boolean;
}

const records = new Map<string, StreamRecord>();
let listenPromise: Promise<UnlistenFn> | null = null;

export const agentStreamManager = { startSession, stopSession, failSession,
  getSnapshot, isStreaming, subscribe };

function ensureListener() {
  if (!listenPromise) {
    listenPromise = listen<StreamEnvelope>(EVENT_NAME, (event) => {
      if (!event.payload?.sessionId) return;
      handleStreamEvent(event.payload.sessionId, event.payload.event);
    });
  }
  return listenPromise;
}

async function startSession(sessionId: string, messages: AgentMessage[], tokenCount: number) {
  await ensureListener();
  const record = getOrCreateRecord(sessionId);
  clearCleanup(record);
  record.state = createManagedStreamState(messages, tokenCount);
  record.history = [];
  record.started = true;
  touchSession(sessionId, record);
  notify(record);
}

function stopSession(sessionId: string) {
  const record = records.get(sessionId);
  if (!record) return;
  const result = finishPartialStream(record.state);
  record.state = result.state;
  notify(record);
  if (result.assistantMessage && !record.state.persisted) {
    persistAssistant(sessionId, record, result.assistantMessage, 0);
  }
}

function failSession(sessionId: string) {
  const record = records.get(sessionId);
  if (!record) return;
  record.state = { ...record.state, isStreaming: false, completed: true,
    error: i18n.t("errors.streamStartFailed"), updatedAt: Date.now() };
  notify(record);
  scheduleCleanup(sessionId, record);
}

function getSnapshot(sessionId: string): StreamSnapshot | null {
  const record = records.get(sessionId);
  if (!record?.started) return null;
  return snapshot(record.state);
}

function isStreaming(sessionId: string): boolean {
  return records.get(sessionId)?.state.isStreaming ?? false;
}

function subscribe(sessionId: string, subscriber: Subscriber): () => void {
  ensureListener();
  const record = getOrCreateRecord(sessionId);
  clearCleanup(record);
  const id = record.nextSubscriberId++;
  record.subscribers.set(id, subscriber);
  trimSubscribers(record);
  if (record.started) subscriber(snapshot(record.state));
  return () => {
    record.subscribers.delete(id);
    if (record.state.completed && record.subscribers.size === 0) {
      scheduleCleanup(sessionId, record);
    }
  };
}

function handleStreamEvent(sessionId: string, event: StreamEvent) {
  const record = getOrCreateRecord(sessionId);
  clearCleanup(record);
  record.started = true;
  record.history.push(event);
  if (record.history.length > MAX_EVENTS_PER_SESSION) {
    record.history.splice(0, record.history.length - MAX_EVENTS_PER_SESSION);
  }
  const result = applyStreamEvent(record.state, event);
  record.state = result.state;
  touchSession(sessionId, record);
  notify(record);
  if (result.assistantMessage && !record.state.persisted) {
    persistAssistant(sessionId, record, result.assistantMessage, result.assistantTokens ?? 0);
  }
  if (record.state.completed && record.subscribers.size === 0) {
    scheduleCleanup(sessionId, record);
  }
}

function persistAssistant(
  sessionId: string, record: StreamRecord, message: AgentMessage, tokens: number,
) {
  record.state = { ...record.state, persisted: true };
  invoke("add_messages_to_session", {
    id: sessionId,
    messages: [message],
    tokens,
  }).catch(() => {
    record.state = { ...record.state, persisted: false };
  });
}

function getOrCreateRecord(sessionId: string): StreamRecord {
  let record = records.get(sessionId);
  if (record) return record;
  record = {
    state: { ...createManagedStreamState([], 0), isStreaming: false },
    history: [],
    subscribers: new Map(),
    nextSubscriberId: 1,
    cleanupTimer: null,
    started: false,
  };
  records.set(sessionId, record);
  enforceSessionLimit();
  return record;
}

function notify(record: StreamRecord) {
  if (!record.started) return;
  const value = snapshot(record.state);
  for (const subscriber of record.subscribers.values()) subscriber(value);
}

function snapshot(state: ManagedStreamState): StreamSnapshot {
  return {
    ...toChatState(state), pendingPermissions: [...state.pendingPermissions],
    completed: state.completed, error: state.error,
    isConnectionError: state.isConnectionError,
  };
}

function trimSubscribers(record: StreamRecord) {
  while (record.subscribers.size > MAX_SUBSCRIBERS_PER_SESSION) {
    const first = record.subscribers.keys().next().value;
    if (first === undefined) break;
    record.subscribers.delete(first);
  }
}

function touchSession(sessionId: string, record: StreamRecord) {
  records.delete(sessionId);
  records.set(sessionId, record);
  enforceSessionLimit();
}

function enforceSessionLimit() {
  for (const [sessionId, record] of records) {
    if (records.size <= MAX_SESSIONS) return;
    if (record.state.isStreaming || record.subscribers.size > 0) continue;
    clearCleanup(record); records.delete(sessionId);
  }
}

function scheduleCleanup(sessionId: string, record: StreamRecord) {
  clearCleanup(record);
  record.cleanupTimer = setTimeout(() => {
    if (record.subscribers.size === 0 && !record.state.isStreaming) {
      records.delete(sessionId);
    }
  }, CLEANUP_DELAY_MS);
}

function clearCleanup(record: StreamRecord) {
  if (record.cleanupTimer) clearTimeout(record.cleanupTimer);
  record.cleanupTimer = null;
}

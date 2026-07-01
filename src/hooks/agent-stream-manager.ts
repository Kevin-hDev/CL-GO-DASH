import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import i18n from "@/i18n";
import {
  applyStreamEvent,
  createManagedStreamState,
  finishPartialStream,
} from "./agent-chat-stream-callbacks";
import {
  MAX_EVENTS_PER_SESSION,
  scheduleCleanup, clearCleanup, trimSubscribers,
  type StreamRecord,
} from "./agent-stream-cleanup";
import {
  flushFrameNotify,
  scheduleFrameNotify,
  shouldDeferStreamEvent,
} from "./agent-stream-notify";
import {
  getOrCreateRecord,
  getRecord,
  persistAssistant,
  records,
  snapshot,
  touchSession,
  type StreamSnapshot,
} from "./agent-stream-records";
import { estimateAgentMessagesTokens } from "./agent-token-estimate";
import { emitStreamActivity, subscribeStreamActivity, toStreamActivity } from "./agent-stream-activity";
import { showToast } from "@/lib/toast-emitter";
import type { AgentMessage, StreamEvent } from "@/types/agent";
import { webToolErrorToastMessage } from "./web-tool-error-toast";

export type { StreamSnapshot } from "./agent-stream-records";

const EVENT_NAME = "agent-stream-event";

interface StreamEnvelope { sessionId: string; event: StreamEvent }

type Subscriber = (snapshot: StreamSnapshot) => void;

let listenPromise: Promise<UnlistenFn> | null = null;

export const agentStreamManager = { startSession, stopSession, failSession,
  getSnapshot, getActivity, isStreaming, subscribe, subscribeActivity: subscribeStreamActivity };

function ensureListener() {
  if (!listenPromise) {
    listenPromise = listen<StreamEnvelope>(EVENT_NAME, (event) => {
      if (!event.payload?.sessionId) return;
      handleStreamEvent(event.payload.sessionId, event.payload.event);
    });
  }
  return listenPromise;
}

async function startSession(sessionId: string, messages: AgentMessage[], sessionTokenCount: number) {
  await ensureListener();
  const record = getOrCreateRecord(sessionId);
  clearCleanup(record);
  record.state = createManagedStreamState(messages, sessionTokenCount);
  record.history = [];
  record.started = true;
  record.isGateway = false; // session UI explicite — le frontend gère la persistance
  touchSession(sessionId, record);
  flushFrameNotify(record, notify);
  notifyActivity(sessionId, record);
}

function stopSession(sessionId: string) {
  const record = getRecord(sessionId);
  if (!record) return;
  const result = finishPartialStream(record.state);
  record.state = result.state;
  flushFrameNotify(record, notify);
  notifyActivity(sessionId, record);
  if (result.assistantMessage && !record.state.persisted && !record.isGateway) {
    persistAssistant(sessionId, record, result.assistantMessage, 0, notify);
  }
}

function failSession(sessionId: string) {
  const record = getRecord(sessionId);
  if (!record) return;
  record.state = { ...record.state, isStreaming: false, completed: true,
    error: i18n.t("errors.streamStartFailed"), updatedAt: Date.now() };
  flushFrameNotify(record, notify);
  notifyActivity(sessionId, record);
  scheduleCleanup(sessionId, record, records);
}

function getSnapshot(sessionId: string): StreamSnapshot | null {
  const record = getRecord(sessionId);
  if (!record?.started) return null;
  return snapshot(record.state);
}

function getActivity(sessionId: string) {
  const record = getRecord(sessionId);
  if (!record?.started) return null;
  return toStreamActivity(sessionId, record.state);
}

function isStreaming(sessionId: string): boolean {
  return getRecord(sessionId)?.state.isStreaming ?? false;
}

function subscribe(sessionId: string, subscriber: Subscriber): () => void {
  void ensureListener();
  const record = getOrCreateRecord(sessionId);
  clearCleanup(record);
  const id = record.nextSubscriberId++;
  record.subscribers.set(id, subscriber as (s: unknown) => void);
  trimSubscribers(record);
  if (record.started) subscriber(snapshot(record.state));
  return () => {
    record.subscribers.delete(id);
    if (record.state.completed && record.subscribers.size === 0) {
      scheduleCleanup(sessionId, record, records);
    }
  };
}

function handleStreamEvent(sessionId: string, event: StreamEvent) {
  const record = getOrCreateRecord(sessionId);
  clearCleanup(record);

  // Si le premier event arrive sans que startSession() ait été appelé,
  // c'est une session gateway : le backend a déjà persisté les messages.
  // On marque isGateway=true pour bloquer persistAssistant côté frontend.
  if (!record.started) {
    record.isGateway = true;
    record.started = true;
  }

  if (event.event === "subagentSpawned" || event.event === "subagentCompleted" || event.event === "todoUpdated") {
    flushFrameNotify(record, notify);
    return;
  }

  if (event.event === "sessionSnapshot") {
    record.state = {
      ...record.state,
      messages: event.data.messages,
      sessionTokenCount: estimateAgentMessagesTokens(event.data.messages),
      sessionTokenCountEstimated: true,
      isStreaming: true,
      persisted: false,
      completed: false,
    };
    flushFrameNotify(record, notify);
    notifyActivity(sessionId, record);
    return;
  }

  if (event.event === "notice") {
    showToast(i18n.t(event.data.messageKey), "info");
    return;
  }

  if (!record.state.isStreaming && event.event !== "done" && event.event !== "error") {
    record.state = { ...record.state, isStreaming: true, persisted: false, completed: false };
  }
  record.history.push(event);
  if (record.history.length > MAX_EVENTS_PER_SESSION) {
    record.history.splice(0, record.history.length - MAX_EVENTS_PER_SESSION);
  }

  const toastMessage = webToolErrorToastMessage(sessionId, event);
  if (toastMessage) showToast(toastMessage, "error");

  // Compression terminée : recharger la session depuis le store
  if (event.event === "compressionComplete") {
    invoke<{ messages: AgentMessage[]; accumulated_tokens: number }>(
      "get_agent_session", { id: sessionId },
    ).then((session) => {
      record.state = {
        ...record.state,
        messages: session.messages,
        completedSegments: [],
        currentContent: "",
        currentThinking: "",
        currentTools: [],
        liveTokenCount: 0,
        streamStartedAt: null,
        segmentStartedAt: null,
        isStreaming: false,
        sessionTokenCount: estimateAgentMessagesTokens(session.messages),
        sessionTokenCountEstimated: true,
        persisted: true,
      };
      flushFrameNotify(record, notify);
      notifyActivity(sessionId, record);
    }).catch(() => console.warn("session reload after compression failed"));
    return;
  }

  const result = applyStreamEvent(record.state, event);
  record.state = result.state;
  touchSession(sessionId, record);
  if (shouldDeferStreamEvent(event)) {
    scheduleFrameNotify(record, notify);
  } else {
    flushFrameNotify(record, notify);
  }
  notifyActivity(sessionId, record);

  // Pour les sessions gateway, le backend persiste déjà — skip persistAssistant.
  // Pour les sessions UI normales, persistAssistant comme avant.
  if (result.assistantMessage && !record.state.persisted && !record.isGateway) {
    persistAssistant(sessionId, record, result.assistantMessage, result.assistantTokens ?? 0, notify);
  }

  if (record.state.completed && record.subscribers.size === 0) {
    scheduleCleanup(sessionId, record, records);
  }
}

function notify(record: StreamRecord) {
  if (!record.started) return;
  const value = snapshot(record.state);
  for (const subscriber of record.subscribers.values()) (subscriber as Subscriber)(value);
}

function notifyActivity(sessionId: string, record: StreamRecord) {
  if (!record.started) return;
  emitStreamActivity(sessionId, record.state);
}

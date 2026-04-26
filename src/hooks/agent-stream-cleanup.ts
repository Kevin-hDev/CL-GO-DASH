import type { ManagedStreamState } from "./agent-chat-stream-callbacks";

const CLEANUP_DELAY_MS = 5 * 60 * 1000;

export const MAX_SESSIONS = 64;
export const MAX_EVENTS_PER_SESSION = 4096;
export const MAX_SUBSCRIBERS_PER_SESSION = 32;

export interface StreamRecord {
  state: ManagedStreamState;
  history: import("@/types/agent").StreamEvent[];
  subscribers: Map<number, (snapshot: unknown) => void>;
  nextSubscriberId: number;
  cleanupTimer: ReturnType<typeof setTimeout> | null;
  started: boolean;
}

export function enforceSessionLimit(records: Map<string, StreamRecord>) {
  for (const [sessionId, record] of records) {
    if (records.size <= MAX_SESSIONS) return;
    if (record.state.isStreaming || record.subscribers.size > 0) continue;
    clearCleanup(record);
    records.delete(sessionId);
  }
}

export function scheduleCleanup(
  sessionId: string,
  record: StreamRecord,
  records: Map<string, StreamRecord>,
) {
  clearCleanup(record);
  record.cleanupTimer = setTimeout(() => {
    if (record.subscribers.size === 0 && !record.state.isStreaming) {
      records.delete(sessionId);
    }
  }, CLEANUP_DELAY_MS);
}

export function clearCleanup(record: StreamRecord) {
  if (record.cleanupTimer) clearTimeout(record.cleanupTimer);
  record.cleanupTimer = null;
}

export function trimSubscribers(record: StreamRecord) {
  while (record.subscribers.size > MAX_SUBSCRIBERS_PER_SESSION) {
    const first = record.subscribers.keys().next().value;
    if (first === undefined) break;
    record.subscribers.delete(first);
  }
}

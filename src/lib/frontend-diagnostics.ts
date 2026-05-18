import { invoke } from "@tauri-apps/api/core";

interface DiagnosticEntry {
  event: string;
  data?: Record<string, unknown>;
  at: string;
}

const MAX_ENTRIES = 80;
const MAX_TEXT = 3000;
const STORAGE_KEY = "clgo-diagnostics";
const SENSITIVE_FIELD_NAMES = ["key", "token", "secret", "password", "authorization"];

function sanitize(value: unknown): unknown {
  if (Array.isArray(value)) return value.slice(0, 20).map(sanitize);
  if (!value || typeof value !== "object") return value;

  return Object.fromEntries(
    Object.entries(value as Record<string, unknown>).map(([key, item]) => [
      key,
      SENSITIVE_FIELD_NAMES.some((needle) => key.toLowerCase().includes(needle))
        ? "[redacted]"
        : sanitize(item),
    ]),
  );
}

function serialize(entry: DiagnosticEntry): string {
  const text = JSON.stringify(entry);
  return text.length > MAX_TEXT ? `${text.slice(0, MAX_TEXT)}...` : text;
}

function readEntries(): DiagnosticEntry[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    const parsed = raw ? JSON.parse(raw) as unknown : [];
    return Array.isArray(parsed) ? parsed.slice(-MAX_ENTRIES) as DiagnosticEntry[] : [];
  } catch {
    return [];
  }
}

function writeEntry(entry: DiagnosticEntry) {
  try {
    const entries = [...readEntries(), entry].slice(-MAX_ENTRIES);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(entries));
  } catch {
    // Diagnostic storage must never affect the app.
  }
}

export function recordFrontendDiagnostic(event: string, data?: Record<string, unknown>) {
  if (!import.meta.env.DEV) return;

  const entry = {
    event,
    data: sanitize(data) as Record<string, unknown> | undefined,
    at: new Date().toISOString(),
  };
  writeEntry(entry);
  void invoke("frontend_diagnostic_log", { entry: serialize(entry) }).catch(() => {});
}

export function getRecentFrontendDiagnostics(): DiagnosticEntry[] {
  return readEntries();
}

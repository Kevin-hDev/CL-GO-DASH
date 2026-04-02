import { invoke } from "@tauri-apps/api/core";
import type { SessionMeta, SessionDetail } from "@/types/session";

export async function listSessions(
  limit: number,
  offset: number,
): Promise<SessionMeta[]> {
  return invoke<SessionMeta[]>("list_sessions", { limit, offset });
}

export async function getSessionDetail(
  sessionId: string,
): Promise<SessionDetail> {
  return invoke<SessionDetail>("get_session_detail", {
    sessionId,
  });
}

export async function renameSession(
  sessionId: string,
  name: string,
): Promise<void> {
  return invoke("rename_session", { sessionId, name });
}

export async function deleteSessionFile(
  filePath: string,
): Promise<void> {
  return invoke("delete_session_file", { filePath });
}

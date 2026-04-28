import { invoke } from "@tauri-apps/api/core";
import type { PreviewEditor } from "@/types/file-preview";

export function readFilePreview(path: string, baseDir?: string) {
  return invoke<string>("read_file_preview", { path, baseDir });
}

export function detectPreviewEditors() {
  return invoke<PreviewEditor[]>("detect_preview_editors");
}

export function openPreviewFile(path: string, baseDir?: string) {
  return invoke("open_preview_file", { path, baseDir });
}

export function openPreviewWithEditor(path: string, editor: string, baseDir?: string) {
  return invoke("open_preview_with_editor", { path, editor, baseDir });
}

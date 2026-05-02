import { invoke } from "@tauri-apps/api/core";
import type { PreviewEditor } from "@/types/file-preview";

export function readFilePreview(path: string, baseDir?: string) {
  return invoke<string>("read_file_preview", { path, baseDir });
}

export function detectEditorsForFile(path: string, baseDir?: string) {
  return invoke<PreviewEditor[]>("detect_editors_for_file", { path, baseDir });
}

export function openPreviewFile(path: string, baseDir?: string) {
  return invoke("open_preview_file", { path, baseDir });
}

export function openPreviewWithEditor(path: string, editorPath: string, baseDir?: string) {
  return invoke("open_preview_with_editor", { path, editorPath, baseDir });
}

export function readSpreadsheetPreview(
  path: string,
  baseDir?: string,
  sheet?: string,
  maxRows?: number,
) {
  return invoke<string>("read_spreadsheet_preview", { path, baseDir, sheet, maxRows });
}

export function readBinaryPreview(path: string, baseDir?: string) {
  return invoke<string>("read_binary_preview", { path, baseDir });
}

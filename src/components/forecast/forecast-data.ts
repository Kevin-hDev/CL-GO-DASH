import { invoke } from "@tauri-apps/api/core";

const MAX_ROWS = 5000;

export interface ForecastDraftData {
  sourceName: string;
  columns: string[];
  rowCount: number;
  dataJson: string;
}

interface SpreadsheetPreview {
  headers: string[];
  rows: unknown[][];
  total_rows: number;
}

export async function loadForecastDraftFromFile(path: string): Promise<ForecastDraftData> {
  const raw = await invoke<string>("read_spreadsheet_preview", {
    path,
    maxRows: MAX_ROWS,
  });
  const preview = JSON.parse(raw) as SpreadsheetPreview;
  const columns = preview.headers.filter((h) => h.trim().length > 0);
  const records = preview.rows.slice(0, MAX_ROWS).map((row) => {
    const item: Record<string, unknown> = {};
    columns.forEach((column, index) => {
      item[column] = normalizeCell(row[index]);
    });
    return item;
  });

  return {
    sourceName: basename(path),
    columns,
    rowCount: records.length || preview.total_rows,
    dataJson: JSON.stringify(records),
  };
}

function normalizeCell(value: unknown): unknown {
  if (typeof value !== "string") return value;
  const trimmed = value.trim();
  if (trimmed === "") return "";
  const normalized = trimmed.replace(",", ".");
  if (isPlainNumber(normalized)) {
    const numeric = Number(normalized);
    if (Number.isFinite(numeric)) return numeric;
  }
  return trimmed;
}

function isPlainNumber(value: string): boolean {
  let dotCount = 0;
  return value.split("").every((char, index) => {
    if (char === "-" && index === 0) return true;
    if (char === ".") {
      dotCount += 1;
      return dotCount <= 1;
    }
    return char >= "0" && char <= "9";
  });
}

function basename(path: string): string {
  return path.split(/[\\/]/).pop() || "data";
}

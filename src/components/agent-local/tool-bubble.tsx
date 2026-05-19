import type { ToolActivity } from "@/hooks/agent-chat-utils";
import type { ToolActivityRecord } from "@/types/agent";
import { ContentPreview, DiffPreview, WebResultsPreview } from "./tool-previews";
import { ReadSpreadsheetPreview, WriteSpreadsheetPreview, DocumentResultPreview, WriteDocumentPreview } from "./tool-office-previews";
import { ToolItem } from "./tool-item";
import "./tool-bubble.css";

function parseLineFromResult(result?: string): number | undefined {
  if (!result) return undefined;
  const match = /\(ligne (\d+)\)/.exec(result);
  return match ? Number(match[1]) : undefined;
}

function str(v: unknown, fallback = ""): string {
  return typeof v === "string" ? v : fallback;
}

function toolSummary(t: ToolActivity): string {
  const a = t.args;
  if (t.name === "bash") return str(a.command);
  if (t.name === "grep") return str(a.pattern);
  if (t.name === "glob") return str(a.pattern);
  if (t.name === "read_file" || t.name === "write_file") return str(a.path);
  if (t.name === "edit_file") return str(a.path);
  if (t.name === "list_dir") return str(a.path, ".");
  if (t.name === "web_search") return str(a.query);
  if (t.name === "web_fetch") return str(a.url);
  if (t.name === "create_branch") return str(a.branch_name);
  if (t.name === "checkout_branch") return str(a.branch_name);
  if (t.name === "read_spreadsheet") return str(a.path);
  if (t.name === "read_document") return str(a.path);
  if (t.name === "read_image") return str(a.path);
  if (t.name === "write_spreadsheet") return str(a.path);
  if (t.name === "write_document") return str(a.path);
  if (t.name === "process_image") return str(a.input_path);
  return JSON.stringify(a).slice(0, 80);
}

export function ToolBubble({
  tools, onFilePreview,
}: { tools: ToolActivity[]; onFilePreview?: (path: string) => void }) {
  if (tools.length === 0) return null;
  return (
    <div className="chat-bubble">
      <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
        {tools.map((t, i) => {
          const skipWrite = t.name === "write_file" && i > 0
            && tools.slice(0, i).some((p) => p.name === "edit_file" && String(p.args.path) === String(t.args.path));
          return (
            <ToolItem key={i} name={t.name} summary={toolSummary(t)} done={!!t.result} isError={t.isError} errorMessage={t.isError ? t.result : undefined} result={t.result} onFilePreview={onFilePreview}>
              {t.name === "write_file" && !skipWrite && typeof t.args.content === "string" && <ContentPreview content={t.args.content} path={toolSummary(t)} />}
              {t.name === "edit_file" && typeof t.args.old_string === "string" && <DiffPreview oldText={t.args.old_string} newText={str(t.args.new_string)} path={toolSummary(t)} startLine={parseLineFromResult(t.result)} />}
              {(t.name === "web_search" || t.name === "web_fetch") && t.result && <WebResultsPreview content={t.result} isSearch={t.name === "web_search"} />}
              {t.name === "read_spreadsheet" && t.result && !t.isError && <ReadSpreadsheetPreview result={t.result} />}
              {t.name === "read_document" && t.result && !t.isError && <DocumentResultPreview result={t.result} />}
              {t.name === "write_spreadsheet" && t.result && !t.isError && <WriteSpreadsheetPreview operations={t.args.operations} />}
              {t.name === "write_document" && t.result && !t.isError && <WriteDocumentPreview content={t.args.content} />}
            </ToolItem>
          );
        })}
      </div>
    </div>
  );
}

export function SavedToolBubble({
  tools, onFilePreview,
}: { tools: ToolActivityRecord[]; onFilePreview?: (path: string) => void }) {
  if (tools.length === 0) return null;
  return (
    <div className="chat-bubble">
      <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
        {tools.map((t, i) => {
          const skipWrite = t.name === "write_file" && i > 0
            && tools.slice(0, i).some((p) => p.name === "edit_file" && p.summary === t.summary);
          return (
            <ToolItem key={i} name={t.name} summary={t.summary} done={t.result != null || t.is_error != null} isError={t.is_error} errorMessage={t.is_error ? t.result : undefined} result={t.result} onFilePreview={onFilePreview}>
              {t.name === "write_file" && t.content && !skipWrite && <ContentPreview content={t.content} path={t.summary} />}
              {t.old_text != null && t.new_text != null && <DiffPreview oldText={t.old_text} newText={t.new_text} path={t.summary} startLine={t.start_line} />}
              {(t.name === "web_search" || t.name === "web_fetch") && t.result && <WebResultsPreview content={t.result} isSearch={t.name === "web_search"} />}
              {t.name === "read_spreadsheet" && t.result && !t.is_error && <ReadSpreadsheetPreview result={t.result} />}
              {t.name === "read_document" && t.result && !t.is_error && <DocumentResultPreview result={t.result} />}
              {t.name === "write_spreadsheet" && t.result && !t.is_error && t.content && <WriteSpreadsheetPreview operations={t.content} />}
              {t.name === "write_document" && t.result && !t.is_error && t.content && <WriteDocumentPreview content={t.content} />}
            </ToolItem>
          );
        })}
      </div>
    </div>
  );
}

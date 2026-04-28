import { useMemo } from "react";
import { highlightLines } from "@/lib/highlight";
import { shouldWrapFile } from "@/lib/code-language";
import type { FileOperation } from "@/types/file-preview";
import "@/components/agent-local/tool-previews.css";

function findStartLine(content: string, needle: string): number {
  if (!needle) return 1;
  const idx = content.indexOf(needle);
  if (idx < 0) return 1;
  return content.slice(0, idx).split("\n").length;
}

export function FilePreviewDiff({
  operation,
  currentContent,
}: {
  operation: FileOperation;
  currentContent: string;
}) {
  const oldLines = useMemo(
    () => highlightLines(operation.oldText ?? "", operation.path),
    [operation.oldText, operation.path],
  );
  const newLines = useMemo(
    () => highlightLines(operation.newText ?? "", operation.path),
    [operation.newText, operation.path],
  );

  const startLine = useMemo(
    () => operation.startLine ?? findStartLine(currentContent, operation.newText ?? operation.oldText ?? ""),
    [currentContent, operation.startLine, operation.newText, operation.oldText],
  );

  const wrap = shouldWrapFile(operation.path);
  const inner = (
    <>
      {oldLines.map((html, i) => (
        <div key={`old-${i}`} className="tp-line tp-line-error">
          <span className="tp-num">{startLine + i}</span>
          <span className="tp-prefix tp-prefix-error">-</span>
          <span className="tp-code tp-code-error" dangerouslySetInnerHTML={{ __html: html || " " }} />
        </div>
      ))}
      {newLines.map((html, i) => (
        <div key={`new-${i}`} className="tp-line tp-line-ok">
          <span className="tp-num">{startLine + i}</span>
          <span className="tp-prefix tp-prefix-ok">+</span>
          <span className="tp-code tp-code-ok" dangerouslySetInnerHTML={{ __html: html || " " }} />
        </div>
      ))}
    </>
  );

  if (wrap) {
    return <div className="tp-wrapper" style={{ margin: 0, border: "none", borderRadius: 0 }}>{inner}</div>;
  }
  return (
    <div className="tp-wrapper tp-nowrap" style={{ margin: 0, border: "none", borderRadius: 0 }}>
      <div className="tp-inner">{inner}</div>
    </div>
  );
}

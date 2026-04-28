import { useMemo } from "react";
import { highlightLines } from "@/lib/highlight";
import type { FileOperation } from "@/types/file-preview";
import "@/components/agent-local/tool-previews.css";

export function FilePreviewDiff({
  operation,
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

  return (
    <div className="tp-wrapper" style={{ margin: 0, border: "none", borderRadius: 0 }}>
      {oldLines.map((html, i) => (
        <div key={`old-${i}`} className="tp-line tp-line-error">
          <span className="tp-num">{i + 1}</span>
          <span className="tp-prefix tp-prefix-error">-</span>
          <span className="tp-code tp-code-error" dangerouslySetInnerHTML={{ __html: html || " " }} />
        </div>
      ))}
      {newLines.map((html, i) => (
        <div key={`new-${i}`} className="tp-line tp-line-ok">
          <span className="tp-num">{i + 1}</span>
          <span className="tp-prefix tp-prefix-ok">+</span>
          <span className="tp-code tp-code-ok" dangerouslySetInnerHTML={{ __html: html || " " }} />
        </div>
      ))}
    </div>
  );
}

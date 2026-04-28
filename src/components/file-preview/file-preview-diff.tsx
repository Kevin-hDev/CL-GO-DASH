import type { FileOperation } from "@/types/file-preview";
import "@/components/agent-local/tool-previews.css";

export function FilePreviewDiff({
  operation,
}: {
  operation: FileOperation;
  currentContent: string;
}) {
  const oldLines = (operation.oldText ?? "").split("\n");
  const newLines = (operation.newText ?? "").split("\n");

  return (
    <div className="tp-wrapper" style={{ margin: 0, border: "none", borderRadius: 0 }}>
      {oldLines.map((line, i) => (
        <div key={`old-${i}`} className="tp-line tp-line-error">
          <span className="tp-num">{i + 1}</span>
          <span className="tp-prefix tp-prefix-error">-</span>
          <span className="tp-code tp-code-error">{line}</span>
        </div>
      ))}
      {newLines.map((line, i) => (
        <div key={`new-${i}`} className="tp-line tp-line-ok">
          <span className="tp-num">{i + 1}</span>
          <span className="tp-prefix tp-prefix-ok">+</span>
          <span className="tp-code tp-code-ok">{line}</span>
        </div>
      ))}
    </div>
  );
}

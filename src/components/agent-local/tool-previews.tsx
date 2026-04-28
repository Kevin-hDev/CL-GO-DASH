import { useMemo } from "react";
import { highlightLines } from "@/lib/highlight";
import { shouldWrapFile } from "@/lib/code-language";
import "./tool-previews.css";
import "@/components/file-preview/file-preview-highlight.css";

export function ContentPreview({ content, path }: { content: string; path?: string }) {
  const lines = useMemo(() => path ? highlightLines(content, path) : content.split("\n"), [content, path]);
  const wrap = !path || shouldWrapFile(path);

  return (
    <div className={`tp-wrapper ${wrap ? "" : "tp-nowrap"}`}>
      {lines.map((line, i) => (
        <div key={i} className="tp-line tp-line-ok">
          <span className="tp-num">{i + 1}</span>
          <span className="tp-prefix tp-prefix-ok">+</span>
          {path
            ? <span className="tp-code tp-code-ok" dangerouslySetInnerHTML={{ __html: line || " " }} />
            : <span className="tp-code tp-code-ok">{line}</span>
          }
        </div>
      ))}
    </div>
  );
}

export function DiffPreview({ oldText, newText, path }: { oldText: string; newText: string; path?: string }) {
  const oldLines = useMemo(() => path ? highlightLines(oldText, path) : oldText.split("\n"), [oldText, path]);
  const newLines = useMemo(() => path ? highlightLines(newText, path) : newText.split("\n"), [newText, path]);
  const wrap = !path || shouldWrapFile(path);

  return (
    <div className={`tp-wrapper ${wrap ? "" : "tp-nowrap"}`}>
      {oldLines.map((line, i) => (
        <div key={`old-${i}`} className="tp-line tp-line-error">
          <span className="tp-num">{i + 1}</span>
          <span className="tp-prefix tp-prefix-error">-</span>
          {path
            ? <span className="tp-code tp-code-error" dangerouslySetInnerHTML={{ __html: line || " " }} />
            : <span className="tp-code tp-code-error">{line}</span>
          }
        </div>
      ))}
      {newLines.map((line, i) => (
        <div key={`new-${i}`} className="tp-line tp-line-ok">
          <span className="tp-num">{i + 1}</span>
          <span className="tp-prefix tp-prefix-ok">+</span>
          {path
            ? <span className="tp-code tp-code-ok" dangerouslySetInnerHTML={{ __html: line || " " }} />
            : <span className="tp-code tp-code-ok">{line}</span>
          }
        </div>
      ))}
    </div>
  );
}

export function WebResultsPreview({ content, isSearch }: { content: string; isSearch: boolean }) {
  if (isSearch) {
    const blocks = content.split("\n\n").filter(Boolean);
    return (
      <div className="tp-web-wrapper">
        {blocks.slice(0, 8).map((block, i) => {
          const lines = block.split("\n");
          const title = (lines[0] ?? "").replace(/\*\*/g, "");
          const url = lines[1] ?? "";
          return (
            <div key={i} style={{ marginBottom: i < blocks.length - 1 ? 6 : 0 }}>
              <div className="tp-web-title">{title}</div>
              <div className="tp-web-url">{url}</div>
            </div>
          );
        })}
      </div>
    );
  }
  const preview = content.length > 300 ? content.slice(0, 300) + "..." : content;
  return (
    <div className="tp-web-fetch">
      {preview}
    </div>
  );
}

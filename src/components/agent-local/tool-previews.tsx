import { useMemo } from "react";
import { highlightLines } from "@/lib/highlight";
import { shouldWrapFile } from "@/lib/code-language";
import "./tool-previews.css";
import "@/components/file-preview/file-preview-highlight.css";

function CodeLines({ lines, mode, path }: { lines: string[]; mode: "ok" | "error"; path?: string }) {
  return (
    <>
      {lines.map((line, i) => (
        <div key={`${mode}-${i}`} className={`tp-line tp-line-${mode}`}>
          <span className="tp-num">{i + 1}</span>
          <span className={`tp-prefix tp-prefix-${mode}`}>{mode === "ok" ? "+" : "-"}</span>
          {path
            ? <span className={`tp-code tp-code-${mode}`} dangerouslySetInnerHTML={{ __html: line || " " }} />
            : <span className={`tp-code tp-code-${mode}`}>{line}</span>
          }
        </div>
      ))}
    </>
  );
}

export function ContentPreview({ content, path }: { content: string; path?: string }) {
  const lines = useMemo(() => path ? highlightLines(content, path) : content.split("\n"), [content, path]);
  const wrap = !path || shouldWrapFile(path);

  if (wrap) {
    return (
      <div className="tp-wrapper">
        <CodeLines lines={lines} mode="ok" path={path} />
      </div>
    );
  }

  return (
    <div className="tp-wrapper tp-nowrap">
      <div className="tp-inner">
        <CodeLines lines={lines} mode="ok" path={path} />
      </div>
    </div>
  );
}

export function DiffPreview({ oldText, newText, path }: { oldText: string; newText: string; path?: string }) {
  const oldLines = useMemo(() => path ? highlightLines(oldText, path) : oldText.split("\n"), [oldText, path]);
  const newLines = useMemo(() => path ? highlightLines(newText, path) : newText.split("\n"), [newText, path]);
  const wrap = !path || shouldWrapFile(path);

  if (wrap) {
    return (
      <div className="tp-wrapper">
        <CodeLines lines={oldLines} mode="error" path={path} />
        <CodeLines lines={newLines} mode="ok" path={path} />
      </div>
    );
  }

  return (
    <div className="tp-wrapper tp-nowrap">
      <div className="tp-inner">
        <CodeLines lines={oldLines} mode="error" path={path} />
        <CodeLines lines={newLines} mode="ok" path={path} />
      </div>
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

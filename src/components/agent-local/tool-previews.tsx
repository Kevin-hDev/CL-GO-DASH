import "./tool-previews.css";

export function ContentPreview({ content }: { content: string }) {
  const lines = content.split("\n");
  return (
    <div className="tp-wrapper">
      {lines.map((line, i) => (
        <div key={i} className="tp-line tp-line-ok">
          <span className="tp-num">{i + 1}</span>
          <span className="tp-prefix tp-prefix-ok">+</span>
          <span className="tp-code tp-code-ok">{line}</span>
        </div>
      ))}
    </div>
  );
}

export function DiffPreview({ oldText, newText }: { oldText: string; newText: string }) {
  const oldLines = oldText.split("\n");
  const newLines = newText.split("\n");
  return (
    <div className="tp-wrapper">
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

const LINE_STYLE: React.CSSProperties = {
  display: "flex",
  fontFamily: "var(--font-mono, monospace)",
  fontSize: "var(--text-xs, 11px)",
  lineHeight: 1.7,
};

const NUM_STYLE: React.CSSProperties = {
  width: 32, textAlign: "right", paddingRight: 8,
  color: "var(--ink-muted)", userSelect: "none", flexShrink: 0,
};

const PREFIX_STYLE: React.CSSProperties = {
  width: 16, textAlign: "center", flexShrink: 0, userSelect: "none",
};

const CODE_STYLE: React.CSSProperties = {
  flex: 1, whiteSpace: "pre-wrap", wordBreak: "break-all",
  paddingRight: 8,
};

export function ContentPreview({ content }: { content: string }) {
  const lines = content.split("\n");
  return (
    <div style={{
      marginTop: 6, borderRadius: 4, overflow: "hidden",
      border: "1px solid rgba(255,255,255,0.06)",
    }}>
      {lines.map((line, i) => (
        <div key={i} style={{ ...LINE_STYLE, background: "rgba(34, 197, 94, 0.15)" }}>
          <span style={NUM_STYLE}>{i + 1}</span>
          <span style={{ ...PREFIX_STYLE, color: "var(--signal-ok)" }}>+</span>
          <span style={{ ...CODE_STYLE, color: "var(--signal-ok)" }}>{line}</span>
        </div>
      ))}
    </div>
  );
}

export function DiffPreview({ oldText, newText }: { oldText: string; newText: string }) {
  const oldLines = oldText.split("\n");
  const newLines = newText.split("\n");
  return (
    <div style={{
      marginTop: 6, borderRadius: 4, overflow: "hidden",
      border: "1px solid rgba(255,255,255,0.06)",
    }}>
      {oldLines.map((line, i) => (
        <div key={`old-${i}`} style={{ ...LINE_STYLE, background: "rgba(220, 38, 38, 0.15)" }}>
          <span style={NUM_STYLE}>{i + 1}</span>
          <span style={{ ...PREFIX_STYLE, color: "var(--signal-error)" }}>-</span>
          <span style={{ ...CODE_STYLE, color: "var(--signal-error)" }}>{line}</span>
        </div>
      ))}
      {newLines.map((line, i) => (
        <div key={`new-${i}`} style={{ ...LINE_STYLE, background: "rgba(34, 197, 94, 0.15)" }}>
          <span style={NUM_STYLE}>{i + 1}</span>
          <span style={{ ...PREFIX_STYLE, color: "var(--signal-ok)" }}>+</span>
          <span style={{ ...CODE_STYLE, color: "var(--signal-ok)" }}>{line}</span>
        </div>
      ))}
    </div>
  );
}

export function WebResultsPreview({ content, isSearch }: { content: string; isSearch: boolean }) {
  if (isSearch) {
    const blocks = content.split("\n\n").filter(Boolean);
    return (
      <div style={{
        marginTop: 6, padding: "6px 8px", borderRadius: 4,
        background: "rgba(155, 127, 255, 0.06)",
        border: "1px solid rgba(155, 127, 255, 0.12)",
        fontSize: "11px", fontFamily: "var(--font-mono, monospace)",
      }}>
        {blocks.slice(0, 8).map((block, i) => {
          const lines = block.split("\n");
          const title = (lines[0] ?? "").replace(/\*\*/g, "");
          const url = lines[1] ?? "";
          return (
            <div key={i} style={{ marginBottom: i < blocks.length - 1 ? 6 : 0 }}>
              <div style={{ color: "var(--pulse)", fontWeight: 600 }}>{title}</div>
              <div style={{ color: "var(--ink-faint)", fontSize: "10px", wordBreak: "break-all" }}>{url}</div>
            </div>
          );
        })}
      </div>
    );
  }
  const preview = content.length > 300 ? content.slice(0, 300) + "..." : content;
  return (
    <div style={{
      marginTop: 6, padding: "6px 8px", borderRadius: 4,
      background: "rgba(155, 127, 255, 0.06)",
      border: "1px solid rgba(155, 127, 255, 0.12)",
      fontSize: "10px", color: "var(--ink-muted)",
      whiteSpace: "pre-wrap", wordBreak: "break-all",
      maxHeight: 150, overflow: "hidden",
    }}>
      {preview}
    </div>
  );
}

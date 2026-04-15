interface DiffViewProps {
  oldText: string;
  newText: string;
}

const LINE_STYLE: React.CSSProperties = {
  display: "flex",
  fontFamily: "var(--font-mono)",
  fontSize: "var(--text-xs)",
  lineHeight: 1.7,
};

const NUM_STYLE: React.CSSProperties = {
  width: 32, textAlign: "right", paddingRight: 8,
  color: "#555", userSelect: "none", flexShrink: 0,
};

const PREFIX_STYLE: React.CSSProperties = {
  width: 16, textAlign: "center", flexShrink: 0, userSelect: "none",
};

const CODE_STYLE: React.CSSProperties = {
  flex: 1, whiteSpace: "pre-wrap", wordBreak: "break-all",
  paddingRight: 8,
};

export function DiffView({ oldText, newText }: DiffViewProps) {
  const oldLines = oldText.split("\n");
  const newLines = newText.split("\n");

  return (
    <div style={{
      marginTop: 6, borderRadius: 4, overflow: "hidden",
      border: "1px solid rgba(255,255,255,0.06)",
      fontSize: "var(--text-xs)",
    }}>
      {/* Removed lines */}
      {oldLines.map((line, i) => (
        <div key={`old-${i}`} style={{
          ...LINE_STYLE,
          background: "rgba(220, 38, 38, 0.15)",
        }}>
          <span style={NUM_STYLE}>{i + 1}</span>
          <span style={{ ...PREFIX_STYLE, color: "#ef4444" }}>-</span>
          <span style={{ ...CODE_STYLE, color: "#fca5a5" }}>{line}</span>
        </div>
      ))}
      {/* Added lines */}
      {newLines.map((line, i) => (
        <div key={`new-${i}`} style={{
          ...LINE_STYLE,
          background: "rgba(34, 197, 94, 0.15)",
        }}>
          <span style={NUM_STYLE}>{i + 1}</span>
          <span style={{ ...PREFIX_STYLE, color: "#22c55e" }}>+</span>
          <span style={{ ...CODE_STYLE, color: "#86efac" }}>{line}</span>
        </div>
      ))}
    </div>
  );
}

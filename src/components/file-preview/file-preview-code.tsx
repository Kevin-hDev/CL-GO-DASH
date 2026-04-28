export interface CodeRow {
  id: string;
  lineNumber?: number;
  text: string;
  kind?: "add" | "del" | "fold";
}

export function FilePreviewCode({ rows }: { rows: CodeRow[] }) {
  return (
    <div className="fp-code">
      {rows.map((row) => (
        <div key={row.id} className={`fp-code-row ${row.kind ? `is-${row.kind}` : ""}`}>
          <span className="fp-code-num">{row.lineNumber ?? ""}</span>
          <span className="fp-code-prefix">{prefixFor(row.kind)}</span>
          <span className="fp-code-text">{row.text || " "}</span>
        </div>
      ))}
    </div>
  );
}

function prefixFor(kind?: CodeRow["kind"]) {
  if (kind === "add") return "+";
  if (kind === "del") return "-";
  return "";
}

import "./markdown-viewer.css";

interface MarkdownViewerProps {
  content: string;
  fileName: string;
  onOpenEditor: () => void;
}

export function MarkdownViewer({
  content,
  fileName,
  onOpenEditor,
}: MarkdownViewerProps) {
  return (
    <>
      <div className="md-header">
        <div className="md-title">{fileName}</div>
        <button className="btn" onClick={onOpenEditor}>
          ↗ MWeb
        </button>
      </div>
      <div className="md-scroll">
        <div
          className="md-view"
          dangerouslySetInnerHTML={{ __html: renderMarkdown(content) }}
        />
      </div>
    </>
  );
}

function renderMarkdown(md: string): string {
  return md
    .split("\n\n")
    .map((block) => {
      const trimmed = block.trim();
      if (trimmed.startsWith("# ")) {
        return `<h1>${esc(trimmed.slice(2))}</h1>`;
      }
      if (trimmed.startsWith("## ")) {
        return `<h2>${esc(trimmed.slice(3))}</h2>`;
      }
      if (trimmed.startsWith("### ")) {
        return `<h3>${esc(trimmed.slice(4))}</h3>`;
      }
      if (trimmed.startsWith("> ")) {
        const quote = trimmed
          .split("\n")
          .map((l) => esc(l.replace(/^>\s?/, "")))
          .join("<br>");
        return `<blockquote>${quote}</blockquote>`;
      }
      if (trimmed.startsWith("- ")) {
        const items = trimmed
          .split("\n")
          .map((l) => `<li>${inlineFormat(esc(l.replace(/^-\s/, "")))}</li>`)
          .join("");
        return `<ul>${items}</ul>`;
      }
      return `<p>${inlineFormat(esc(trimmed))}</p>`;
    })
    .join("");
}

function esc(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function inlineFormat(s: string): string {
  return s
    .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
    .replace(/`(.+?)`/g, "<code>$1</code>");
}

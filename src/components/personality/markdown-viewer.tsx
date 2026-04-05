import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { ArrowSquareOut } from "@phosphor-icons/react";
import "./markdown-viewer.css";

interface MarkdownViewerProps {
  content: string;
  fileName: string;
  onOpenEditor: () => void;
}

function stripFrontmatter(md: string): string {
  if (md.startsWith("---")) {
    const end = md.indexOf("---", 3);
    if (end > 0) return md.slice(end + 3).trim();
  }
  return md;
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
          <ArrowSquareOut size={14} /> Open
        </button>
      </div>
      <div className="md-scroll">
        <div className="md-view">
          <ReactMarkdown remarkPlugins={[remarkGfm]}>
            {stripFrontmatter(content)}
          </ReactMarkdown>
        </div>
      </div>
    </>
  );
}

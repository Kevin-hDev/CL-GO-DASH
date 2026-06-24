import { useState, useCallback, useMemo, type ReactNode } from "react";
import { Copy, Check } from "@/components/ui/icons";
import { highlightCodeNodes, type HighlightNode } from "@/lib/highlight";
import "@/components/file-preview/file-preview-highlight.css";
import "./messages.css";

interface CodeBlockProps {
  code: string;
  language?: string;
}

function renderHighlightNode(node: HighlightNode, index: number): ReactNode {
  if (typeof node === "string") return node;
  return (
    <span key={index} className={node.className}>
      {node.children.map(renderHighlightNode)}
    </span>
  );
}

export function CodeBlock({ code, language }: CodeBlockProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    await navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [code]);

  const highlighted = useMemo(
    () => language ? highlightCodeNodes(code, language) : null,
    [code, language],
  );

  return (
    <div className="code-block">
      <div className="code-block-header">
        <span>{language || ""}</span>
        <button className="msg-action-btn" onClick={() => void handleCopy()}>
          {copied ? <Check size={14} /> : <Copy size={14} />}
        </button>
      </div>
      <pre>
        {highlighted
          ? <code>{highlighted.map(renderHighlightNode)}</code>
          : <code>{code}</code>
        }
      </pre>
    </div>
  );
}

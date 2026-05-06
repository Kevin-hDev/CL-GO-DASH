import { useState, useCallback, useMemo } from "react";
import { Copy, Check } from "@/components/ui/icons";
import { highlightCode } from "@/lib/highlight";
import "@/components/file-preview/file-preview-highlight.css";
import "./messages.css";

interface CodeBlockProps {
  code: string;
  language?: string;
}

export function CodeBlock({ code, language }: CodeBlockProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    await navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [code]);

  const html = useMemo(
    () => language ? highlightCode(code, language) : null,
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
        {html
          ? <code dangerouslySetInnerHTML={{ __html: html }} />
          : <code>{code}</code>
        }
      </pre>
    </div>
  );
}

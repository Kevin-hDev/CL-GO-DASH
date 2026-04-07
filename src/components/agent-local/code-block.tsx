import { useState, useCallback } from "react";
import { Copy, Check } from "@/components/ui/icons";
import "./chat.css";

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

  return (
    <div className="code-block">
      <div className="code-block-header">
        <span>{language || ""}</span>
        <button className="msg-action-btn" onClick={handleCopy}>
          {copied ? <Check size={14} /> : <Copy size={14} />}
        </button>
      </div>
      <pre><code>{code}</code></pre>
    </div>
  );
}

import { useState, useCallback } from "react";
import { ArrowsClockwise, Pencil, Copy, Check } from "@/components/ui/icons";
import "./messages.css";

interface MessageActionsProps {
  role: "user" | "assistant";
  content: string;
  isStreaming?: boolean;
  onReload?: () => void;
  onEdit?: () => void;
  children?: React.ReactNode;
}

export function MessageActions({ role, content, isStreaming, onReload, onEdit, children }: MessageActionsProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    await navigator.clipboard.writeText(content);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [content]);

  return (
    <div className="msg-actions">
      {onReload && !isStreaming && (
        <button className="msg-action-btn" onClick={onReload}>
          <ArrowsClockwise size={14} />
        </button>
      )}
      {role === "user" && onEdit && (
        <button className="msg-action-btn" onClick={onEdit}>
          <Pencil size={14} />
        </button>
      )}
      <button className="msg-action-btn" onClick={handleCopy}>
        {copied ? <Check size={14} /> : <Copy size={14} />}
      </button>
      {children}
    </div>
  );
}

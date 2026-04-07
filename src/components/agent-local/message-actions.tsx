import { useState, useCallback } from "react";
import { ArrowsClockwise, Pencil, Copy, Check } from "@/components/ui/icons";
import "./chat.css";

interface MessageActionsProps {
  role: "user" | "assistant";
  content: string;
  onReload?: () => void;
  onEdit?: () => void;
}

export function MessageActions({ role, content, onReload, onEdit }: MessageActionsProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    await navigator.clipboard.writeText(content);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [content]);

  return (
    <div className="msg-actions">
      {onReload && (
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
    </div>
  );
}

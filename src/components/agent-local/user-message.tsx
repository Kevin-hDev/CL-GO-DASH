import { useState } from "react";
import { MessageActions } from "./message-actions";
import { EditMessage } from "./edit-message";
import { useHoverClass } from "@/hooks/use-hover-class";
import "./messages.css";

interface FileInfo {
  name: string;
  path?: string;
  thumbnail?: string;
}

interface UserMessageProps {
  content: string;
  files?: FileInfo[];
  skillNames?: string[];
  isStreaming?: boolean;
  onReload?: () => void;
  onEdit?: (newContent: string) => void;
  onFileClick?: (file: FileInfo) => void;
}

export function UserMessage({
  content, files, skillNames, isStreaming, onReload, onEdit, onFileClick,
}: UserMessageProps) {
  const hoverRef = useHoverClass();
  const [editing, setEditing] = useState(false);

  if (editing && onEdit) {
    return (
      <EditMessage
        initialContent={content}
        onSave={(c) => { onEdit(c); setEditing(false); }}
        onCancel={() => setEditing(false)}
      />
    );
  }

  const hasFiles = files && files.length > 0;
  const hasText = content.trim().length > 0;

  return (
    <div className="msg-user" ref={hoverRef}>
      <div className="msg-user-wrap">
        {(hasText || (skillNames && skillNames.length > 0)) && (
          <div className="msg-user-bubble">
            {hasText && content}
            {skillNames && skillNames.map((name) => (
              <span key={name} style={{
                display: "inline-flex", alignItems: "center", gap: 4,
                padding: "1px 7px 1px 5px", marginLeft: 6,
                borderRadius: "var(--radius-sm)",
                background: "var(--pulse-muted)",
                color: "var(--pulse)",
                fontSize: "var(--text-xs)",
                fontWeight: 500,
                verticalAlign: "middle",
              }}>
                <svg width="12" height="12" viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeWidth="1.5">
                  <rect x="3" y="3" width="14" height="14" rx="3" />
                  <path d="M8 7l4 3-4 3" />
                </svg>
                {name}
              </span>
            ))}
          </div>
        )}
        {hasFiles && (
          <div style={{
            display: "flex", gap: 8, justifyContent: "flex-end",
            flexWrap: "wrap", marginTop: hasText ? 8 : 0,
          }}>
            {files.map((f, i) => (
              <FileCard key={`${f.name}-${i}`} file={f} onClick={() => onFileClick?.(f)} />
            ))}
          </div>
        )}
        <MessageActions
          role="user"
          content={content}
          isStreaming={isStreaming}
          onReload={onReload}
          onEdit={onEdit ? () => setEditing(true) : undefined}
        />
      </div>
    </div>
  );
}

function FileCard({ file, onClick }: { file: FileInfo; onClick: () => void }) {
  const ext = file.name.split(".").pop()?.toLowerCase() ?? "";
  const isImg = ["png", "jpg", "jpeg", "gif", "webp"].includes(ext);

  if (isImg && file.thumbnail) {
    return (
      <div
        onClick={onClick}
        style={{
          width: 100, height: 100, borderRadius: 8,
          overflow: "hidden", cursor: "pointer",
          border: "1px solid var(--edge)",
        }}
      >
        <img src={file.thumbnail} alt="" style={{
          width: "100%", height: "100%", objectFit: "cover",
        }} />
      </div>
    );
  }

  return (
    <div
      onClick={onClick}
      style={{
        width: 100, height: 100, borderRadius: 8,
        border: "1px solid var(--edge)", background: "var(--shell)",
        cursor: "pointer", display: "flex", flexDirection: "column",
        justifyContent: "space-between", padding: 10,
      }}
    >
      <div style={{
        fontSize: "var(--text-xs)", color: "var(--ink)",
        overflow: "hidden", textOverflow: "ellipsis",
        display: "-webkit-box", WebkitLineClamp: 2,
        WebkitBoxOrient: "vertical",
      }}>
        {file.name}
      </div>
      <div style={{
        fontSize: "var(--text-xs)", color: "var(--ink-faint)",
        textTransform: "uppercase", fontWeight: 600,
      }}>
        {ext}
      </div>
    </div>
  );
}

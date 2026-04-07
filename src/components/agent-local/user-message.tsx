import { useState } from "react";
import { MessageActions } from "./message-actions";
import { EditMessage } from "./edit-message";
import { FileText } from "@/components/ui/icons";
import "./messages.css";

interface FileInfo {
  name: string;
  path?: string;
  thumbnail?: string;
}

interface UserMessageProps {
  content: string;
  files?: FileInfo[];
  onReload?: () => void;
  onEdit?: (newContent: string) => void;
  onFileClick?: (file: FileInfo) => void;
}

export function UserMessage({
  content, files, onReload, onEdit, onFileClick,
}: UserMessageProps) {
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

  return (
    <div className="msg-user">
      <MessageActions
        role="user"
        content={content}
        onReload={onReload}
        onEdit={onEdit ? () => setEditing(true) : undefined}
      />
      <div>
        <div className="msg-user-bubble">{content}</div>
        {hasFiles && (
          <div style={{
            display: "flex", gap: 6, marginTop: 6,
            justifyContent: "flex-end", flexWrap: "wrap",
          }}>
            {files.map((f, i) => (
              <div
                key={`${f.name}-${i}`}
                onClick={() => onFileClick?.(f)}
                style={{
                  display: "flex", alignItems: "center", gap: 4,
                  padding: "4px 8px", borderRadius: "var(--radius-sm)",
                  border: "1px solid var(--edge)", background: "var(--shell)",
                  cursor: "pointer", fontSize: "var(--text-xs)",
                  color: "var(--ink-muted)",
                }}
              >
                {f.thumbnail ? (
                  <img src={f.thumbnail} alt={f.name} style={{
                    width: 32, height: 32, objectFit: "cover", borderRadius: 4,
                  }} />
                ) : (
                  <FileText size={14} style={{ color: "var(--ink-faint)" }} />
                )}
                <span style={{
                  maxWidth: 80, overflow: "hidden",
                  textOverflow: "ellipsis", whiteSpace: "nowrap",
                }}>
                  {f.name}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

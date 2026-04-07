import { useState } from "react";
import { MessageActions } from "./message-actions";
import { EditMessage } from "./edit-message";
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
  const hasText = content.trim().length > 0;

  return (
    <div className="msg-user">
      <MessageActions
        role="user"
        content={content}
        onReload={onReload}
        onEdit={onEdit ? () => setEditing(true) : undefined}
      />
      <div>
        {hasText && <div className="msg-user-bubble">{content}</div>}
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

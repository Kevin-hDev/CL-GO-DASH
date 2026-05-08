import { useState, memo } from "react";
import { MessageActions } from "./message-actions";
import { EditMessage } from "./edit-message";
import { useHoverClass } from "@/hooks/use-hover-class";
import { linkifyWithPreviews } from "@/lib/linkify";
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

export const UserMessage = memo(function UserMessage({
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
  const { text: textNodes, previews } = hasText
    ? linkifyWithPreviews(content)
    : { text: [], previews: null };

  return (
    <div className="msg-user" ref={hoverRef}>
      <div className="msg-user-wrap">
        {(hasText || (skillNames && skillNames.length > 0)) && (
          <div className="msg-user-bubble">
            {hasText && textNodes}
            {skillNames && skillNames.map((name) => (
              <span key={name} className="msg-skill-badge">
                <svg width="12" height="12" viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeWidth="1.5">
                  <rect x="3" y="3" width="14" height="14" rx="3" />
                  <path d="M8 7l4 3-4 3" />
                </svg>
                {name}
              </span>
            ))}
            {previews}
          </div>
        )}
        {hasFiles && (
          <div className={`msg-files${hasText ? " msg-files-spaced" : ""}`}>
            {files.map((f, i) => (
              <FileCard key={`${f.name}-${i}`} file={f} onClick={() => onFileClick?.(f)} />
            ))}
          </div>
        )}
        <MessageActions
          messageRole="user"
          content={content}
          isStreaming={isStreaming}
          onReload={onReload}
          onEdit={onEdit ? () => setEditing(true) : undefined}
        />
      </div>
    </div>
  );
});

function FileCard({ file, onClick }: { file: FileInfo; onClick: () => void }) {
  const ext = file.name.split(".").pop()?.toLowerCase() ?? "";
  const isImg = ["png", "jpg", "jpeg", "gif", "webp"].includes(ext);

  if (isImg && file.thumbnail) {
    return (
      <div
        role="button"
        tabIndex={0}
        onClick={onClick}
        onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') onClick(); }}
        className="msg-file-card msg-file-card-img"
      >
        <img src={file.thumbnail} alt="" />
      </div>
    );
  }

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={onClick}
      onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') onClick(); }}
      className="msg-file-card msg-file-card-text"
    >
      <div className="msg-file-name">{file.name}</div>
      <div className="msg-file-ext">{ext}</div>
    </div>
  );
}

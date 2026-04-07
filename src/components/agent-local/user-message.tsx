import { useState } from "react";
import { MessageActions } from "./message-actions";
import { EditMessage } from "./edit-message";
import "./messages.css";

interface UserMessageProps {
  content: string;
  onReload?: () => void;
  onEdit?: (newContent: string) => void;
}

export function UserMessage({ content, onReload, onEdit }: UserMessageProps) {
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

  return (
    <div className="msg-user">
      <MessageActions
        role="user"
        content={content}
        onReload={onReload}
        onEdit={onEdit ? () => setEditing(true) : undefined}
      />
      <div className="msg-user-bubble">{content}</div>
    </div>
  );
}

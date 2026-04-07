import { useState } from "react";
import { useTranslation } from "react-i18next";

interface EditMessageProps {
  initialContent: string;
  onSave: (content: string) => void;
  onCancel: () => void;
}

export function EditMessage({ initialContent, onSave, onCancel }: EditMessageProps) {
  const { t } = useTranslation();
  const [content, setContent] = useState(initialContent);

  return (
    <div style={{
      display: "flex", flexDirection: "column", gap: 8,
      marginBottom: "var(--space-md)", maxWidth: "80%", marginLeft: "auto",
    }}>
      <textarea
        value={content}
        onChange={(e) => setContent(e.target.value)}
        style={{
          width: "100%", borderRadius: "var(--radius-md)",
          padding: "var(--space-sm) var(--space-md)",
          fontSize: "var(--text-sm)", fontFamily: "var(--font-sans)",
          background: "var(--void)", color: "var(--ink)",
          border: "2px solid #3b82f6", resize: "none", outline: "none",
        }}
        rows={3}
      />
      <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
        <button
          onClick={onCancel}
          style={{
            fontSize: "var(--text-xs)", padding: "4px 12px",
            borderRadius: "var(--radius-sm)", border: "1px solid var(--edge)",
            background: "none", color: "var(--ink-muted)", cursor: "pointer",
          }}
        >
          {t("ollama.cancel")}
        </button>
        <button
          onClick={() => onSave(content)}
          style={{
            fontSize: "var(--text-xs)", padding: "4px 12px",
            borderRadius: "var(--radius-sm)", border: "none",
            background: "var(--pulse)", color: "white", cursor: "pointer",
          }}
        >
          {t("ollama.save")}
        </button>
      </div>
    </div>
  );
}

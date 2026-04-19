import { useState, useRef, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";

const MIN_LINES = 3;
const MAX_LINES = 7;
const LINE_HEIGHT = 22;
const BUTTONS_HEIGHT = 52;

interface EditMessageProps {
  initialContent: string;
  onSave: (content: string) => void;
  onCancel: () => void;
}

export function EditMessage({ initialContent, onSave, onCancel }: EditMessageProps) {
  const { t } = useTranslation();
  const [content, setContent] = useState(initialContent);
  const [expanded, setExpanded] = useState(false);
  const [textHeight, setTextHeight] = useState(MIN_LINES * LINE_HEIGHT);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const measureHeight = useCallback(() => {
    const ta = textareaRef.current;
    if (!ta) return;
    ta.style.height = "0px";
    const scroll = ta.scrollHeight;
    const min = MIN_LINES * LINE_HEIGHT;
    const max = MAX_LINES * LINE_HEIGHT;
    const clamped = Math.max(min, Math.min(scroll, max));
    ta.style.height = `${clamped}px`;
    setTextHeight(clamped);
  }, []);

  useEffect(() => {
    measureHeight();
  }, [content, measureHeight]);

  useEffect(() => {
    requestAnimationFrame(() => setExpanded(true));
  }, []);

  useEffect(() => {
    textareaRef.current?.focus();
  }, []);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    const keyMap: Record<string, () => void> = {
      Escape: onCancel,
    };
    const action = keyMap[e.key];
    if (action) action();
    if (e.key.startsWith("Ent") && (e.metaKey || e.ctrlKey)) onSave(content);
  }, [onCancel, onSave, content]);

  const totalHeight = textHeight + BUTTONS_HEIGHT;

  return (
    <div style={{
      marginBottom: "var(--space-md)",
      transition: expanded ? "max-width 700ms cubic-bezier(0.4, 0, 0.2, 1)" : "none",
      maxWidth: expanded ? 720 : "75%",
      marginLeft: "auto",
      marginRight: expanded ? "auto" : 0,
    }}>
      <div style={{
        background: "#1e1e22",
        borderRadius: "18px",
        padding: "var(--space-sm) var(--space-md)",
        overflow: "hidden",
        transition: "height 200ms ease",
        height: expanded ? totalHeight : undefined,
      }}>
        <textarea
          ref={textareaRef}
          value={content}
          onChange={(e) => setContent(e.target.value)}
          onKeyDown={handleKeyDown}
          style={{
            width: "100%",
            border: "none",
            outline: "none",
            resize: "none",
            background: "transparent",
            color: "var(--ink)",
            fontSize: "var(--text-sm)",
            fontFamily: "var(--font-sans)",
            lineHeight: `${LINE_HEIGHT}px`,
            overflow: textHeight >= MAX_LINES * LINE_HEIGHT ? "auto" : "hidden",
          }}
        />
        <div style={{
          display: "flex", gap: 8, justifyContent: "flex-end",
          paddingTop: 4,
        }}>
          <button
            onClick={onCancel}
            style={{
              fontSize: "var(--text-xs)", padding: "5px 14px",
              borderRadius: "var(--radius-md)", border: "1px solid var(--edge)",
              background: "transparent", color: "var(--ink-muted)", cursor: "pointer",
            }}
          >
            {t("agentLocal.cancel")}
          </button>
          <button
            onClick={() => onSave(content)}
            style={{
              fontSize: "var(--text-xs)", padding: "5px 14px",
              borderRadius: "var(--radius-md)", border: "none",
              background: "rgba(255, 255, 255, 0.18)", color: "var(--ink)",
              cursor: "pointer",
            }}
          >
            {t("agentLocal.send")}
          </button>
        </div>
      </div>
    </div>
  );
}

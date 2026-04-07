import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "@/components/ui/icons";
import { useAutoResize } from "@/hooks/use-auto-resize";
import { useSlashCommands } from "@/hooks/use-slash-commands";
import { SendStopButton } from "./send-stop-button";
import { OllamaIndicator } from "./ollama-indicator";
import { SlashAutocomplete } from "./slash-autocomplete";
import { FileThumbnail } from "./file-thumbnail";
import type { DroppedFile } from "@/hooks/use-file-drop";
import "./chat.css";

interface ChatInputProps {
  modelName: string;
  ollamaRunning: boolean;
  isStreaming: boolean;
  files?: DroppedFile[];
  onSend: (text: string) => void;
  onStop: () => void;
  onFileImport: () => void;
  onRemoveFile?: (index: number) => void;
  onPreviewFile?: (file: DroppedFile) => void;
}

export function ChatInput({
  modelName, ollamaRunning, isStreaming, files,
  onSend, onStop, onFileImport, onRemoveFile, onPreviewFile,
}: ChatInputProps) {
  const { t } = useTranslation();
  const [text, setText] = useState("");
  const { ref, resize } = useAutoResize();
  const slash = useSlashCommands();

  const hasContent = text.trim().length > 0 || (files && files.length > 0);

  const handleSend = useCallback(() => {
    if (!hasContent) return;
    onSend(text.trim());
    setText("");
    if (ref.current) ref.current.style.height = "auto";
  }, [text, hasContent, onSend, ref]);

  const handleChange = useCallback((value: string) => {
    setText(value);
    resize();
    slash.handleInput(value);
  }, [resize, slash]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key.startsWith("Ent") && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
    if (e.key.startsWith("Esc")) {
      if (slash.showDropdown) slash.close();
      else onStop();
    }
  }, [handleSend, onStop, slash]);

  const buttonState = isStreaming ? "stop" as const
    : hasContent ? "send" as const
    : "hidden" as const;

  return (
    <div className="chat-input-wrap">
      <div className="chat-input-box">
        {slash.showDropdown && (
          <SlashAutocomplete
            skills={slash.skills}
            onSelect={(skill) => { slash.selectSkill(skill); setText(""); }}
          />
        )}
        <textarea
          ref={ref}
          value={text}
          onChange={(e) => handleChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={t("agentLocal.placeholder")}
          className="chat-textarea"
          rows={1}
        />
        <div style={{ position: "absolute", right: 8, bottom: 8 }}>
          <SendStopButton state={buttonState} onSend={handleSend} onStop={onStop} />
        </div>
      </div>
      {files && files.length > 0 && (
        <div style={{ display: "flex", gap: 8, marginTop: 8, flexWrap: "wrap" }}>
          {files.map((f, i) => (
            <FileThumbnail
              key={`${f.name}-${i}`}
              file={f}
              onRemove={() => onRemoveFile?.(i)}
              onClick={() => onPreviewFile?.(f)}
            />
          ))}
        </div>
      )}
      <div className="chat-input-footer">
        <button className="conv-add-btn" onClick={onFileImport}>
          <Plus size={14} />
        </button>
        <div className="chat-model-info">
          <span>{modelName}</span>
          <OllamaIndicator running={ollamaRunning} />
        </div>
      </div>
    </div>
  );
}

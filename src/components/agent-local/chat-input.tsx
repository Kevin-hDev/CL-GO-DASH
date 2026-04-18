import { useState, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "@/components/ui/icons";
import { useAutoResize } from "@/hooks/use-auto-resize";
import { useSlashCommands } from "@/hooks/use-slash-commands";
import { SendStopButton } from "./send-stop-button";
import { SlashAutocomplete } from "./slash-autocomplete";
import { ModelSelector } from "./model-selector";
import { FileThumbnail } from "./file-thumbnail";
import { ContextProgress } from "./context-progress";
import { PermissionModeSelector } from "./permission-mode-selector";
import type { DroppedFile } from "@/hooks/use-file-drop";
import type { PermissionMode } from "@/hooks/use-permission-mode";
import "./chat.css";

interface ChatInputProps {
  modelName: string;
  providerName: string;
  isStreaming: boolean;
  thinkingEnabled: boolean;
  files?: DroppedFile[];
  contextUsed: number;
  contextMax: number;
  permissionMode: PermissionMode;
  onPermissionModeChange: (mode: PermissionMode) => void;
  onSend: (text: string, files?: DroppedFile[]) => void;
  onStop: () => void;
  onFileImport: () => void;
  onModelChange: (model: string, provider: string) => void;
  onToggleThinking: () => void;
  onSkillLoaded?: (content: string | null) => void;
  onRemoveFile?: (index: number) => void;
  onPreviewFile?: (file: DroppedFile) => void;
  onClearFiles?: () => void;
}

export function ChatInput({
  modelName, providerName, isStreaming, thinkingEnabled, files,
  contextUsed, contextMax,
  permissionMode, onPermissionModeChange,
  onSend, onStop, onFileImport, onModelChange, onToggleThinking, onSkillLoaded,
  onRemoveFile, onPreviewFile, onClearFiles,
}: ChatInputProps) {
  const { t } = useTranslation();
  const [text, setText] = useState("");
  const { ref, resize } = useAutoResize(200);
  const slash = useSlashCommands();

  const hasText = text.trim().length > 0;
  const hasFiles = files != null && files.length > 0;
  const hasContent = hasText || hasFiles;

  const handleSend = useCallback(() => {
    if (!hasContent) return;
    onSend(text.trim(), hasFiles ? files : undefined);
    setText("");
    onClearFiles?.();
    if (ref.current) ref.current.style.height = "auto";
  }, [text, hasContent, hasFiles, files, onSend, onClearFiles, ref]);

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

  useEffect(() => {
    if (!isStreaming) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key.startsWith("Esc")) onStop();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [isStreaming, onStop]);

  const buttonState = isStreaming ? "stop" as const
    : hasContent ? "send" as const
    : "hidden" as const;

  return (
    <div className="chat-input-bubble">
      {slash.showDropdown && (
        <SlashAutocomplete
          skills={slash.skills}
          onSelect={async (skill) => {
            const content = await slash.selectSkill(skill);
            onSkillLoaded?.(content);
            setText("");
          }}
        />
      )}
      <textarea
        ref={ref}
        value={text}
        onChange={(e) => handleChange(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={t("agentLocal.placeholder")}
        className="chat-textarea"
        rows={2}
      />
      {files && files.length > 0 && (
        <div style={{ display: "flex", gap: 6, padding: "0 var(--space-sm)", flexWrap: "wrap" }}>
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
      <div className="chat-input-row3">
        <button className="chat-plus-btn" onClick={onFileImport}>
          <Plus size={16} />
        </button>
        <ContextProgress used={contextUsed} max={contextMax} />
        <PermissionModeSelector mode={permissionMode} onChange={onPermissionModeChange} />
        <ModelSelector
          selectedModel={modelName}
          selectedProvider={providerName}
          onSelect={onModelChange}
          thinkingEnabled={thinkingEnabled}
          onToggleThinking={onToggleThinking}
        />
        <div className="chat-input-spacer" />
        <SendStopButton state={buttonState} onSend={handleSend} onStop={onStop} />
      </div>
    </div>
  );
}

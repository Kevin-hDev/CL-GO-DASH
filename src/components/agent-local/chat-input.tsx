import { useState, useCallback, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "@/components/ui/icons";
import { useAutoResize } from "@/hooks/use-auto-resize";
import { useSlashCommands } from "@/hooks/use-slash-commands";
import { useActiveSkills } from "@/hooks/use-active-skills";
import { SendStopButton } from "./send-stop-button";
import { SlashAutocomplete } from "./slash-autocomplete";
import { SkillBadge } from "./skill-badge";
import { ModelSelector } from "./model-selector";
import { FileThumbnail } from "./file-thumbnail";
import { ContextProgress } from "./context-progress";
import { PermissionModeSelector } from "./permission-mode-selector";
import type { DroppedFile } from "@/hooks/use-file-drop";
import type { PermissionMode } from "@/hooks/use-permission-mode";
import "./chat.css";

const K_UP = "ArrowUp";
const K_DOWN = "ArrowDown";
const K_ENTER = "Enter";
const K_ESC = "Escape";

function eventKey(e: React.KeyboardEvent | KeyboardEvent): string {
  return (e as unknown as Record<string, string>)["key"];
}

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
  onSend: (text: string, files?: DroppedFile[], skills?: { name: string; content: string }[]) => void;
  onStop: () => void;
  onFileImport: () => void;
  onModelChange: (model: string, provider: string) => void;
  onToggleThinking: () => void;
  onRemoveFile?: (index: number) => void;
  onPreviewFile?: (file: DroppedFile) => void;
  onClearFiles?: () => void;
}

export function ChatInput({
  modelName, providerName, isStreaming, thinkingEnabled, files,
  contextUsed, contextMax,
  permissionMode, onPermissionModeChange,
  onSend, onStop, onFileImport, onModelChange, onToggleThinking,
  onRemoveFile, onPreviewFile, onClearFiles,
}: ChatInputProps) {
  const { t } = useTranslation();
  const [text, setText] = useState("");
  const { ref, resize } = useAutoResize(200);
  const slash = useSlashCommands();
  const skills = useActiveSkills(slash, text, setText);
  const bubbleRef = useRef<HTMLDivElement>(null);

  const hasText = text.trim().length > 0;
  const hasFiles = files != null && files.length > 0;
  const hasContent = hasText || hasFiles || skills.activeSkills.length > 0;

  const handleSend = useCallback(() => {
    if (!hasContent || isStreaming) return;
    onSend(text.trim(), hasFiles ? files : undefined, skills.getSkillsPayload());
    setText("");
    skills.clearSkills();
    onClearFiles?.();
    if (ref.current) ref.current.style.height = "auto";
  }, [text, hasContent, hasFiles, files, skills, isStreaming, onSend, onClearFiles, ref]);

  const handleChange = useCallback((value: string) => {
    setText(value);
    resize();
    slash.handleInput(value);
  }, [resize, slash]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    const pressed = eventKey(e);
    if (slash.showDropdown) {
      if (pressed === K_UP) { e.preventDefault(); slash.moveUp(); return; }
      if (pressed === K_DOWN) { e.preventDefault(); slash.moveDown(); return; }
      if (pressed === K_ENTER) {
        e.preventDefault();
        const selected = slash.skills[slash.activeIndex];
        if (selected) skills.handleSelectSkill(selected);
        return;
      }
      if (pressed === K_ESC) { e.preventDefault(); slash.close(); return; }
    }
    if (pressed === K_ENTER && !e.shiftKey) { e.preventDefault(); handleSend(); }
    if (pressed === K_ESC) onStop();
  }, [handleSend, onStop, slash, skills]);

  useEffect(() => {
    if (!isStreaming) return;
    const handler = (e: KeyboardEvent) => { if (eventKey(e) === K_ESC) onStop(); };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [isStreaming, onStop]);

  useEffect(() => {
    if (!slash.showDropdown) return;
    const handler = (e: MouseEvent) => {
      if (bubbleRef.current && !bubbleRef.current.contains(e.target as Node)) slash.close();
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [slash.showDropdown, slash]);

  const buttonState = isStreaming ? "stop" as const
    : hasContent ? "send" as const
    : "hidden" as const;

  return (
    <div className="chat-input-bubble" ref={bubbleRef}>
      {slash.showDropdown && (
        <SlashAutocomplete
          skills={slash.skills}
          activeIndex={slash.activeIndex}
          onSelect={skills.handleSelectSkill}
        />
      )}
      {skills.activeSkills.length > 0 && (
        <div style={{ padding: "var(--space-xs) var(--space-sm) 0", display: "flex", gap: 6, flexWrap: "wrap" }}>
          {skills.activeSkills.map((s) => (
            <SkillBadge key={s.name} skill={s} onRemove={() => skills.handleRemoveSkill(s.name)} />
          ))}
        </div>
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

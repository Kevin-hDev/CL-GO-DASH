import { useState, useCallback, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { ChatPlusMenu } from "./chat-plus-menu";
import { useAutoResize } from "@/hooks/use-auto-resize";
import { useSlashCommands } from "@/hooks/use-slash-commands";
import { useActiveSkills } from "@/hooks/use-active-skills";
import { SendStopButton } from "./send-stop-button";
import { SlashAutocomplete } from "./slash-autocomplete";
import { ModelSelector } from "./model-selector";
import { FileThumbnail } from "./file-thumbnail";
import { ContextProgress } from "./context-progress";
import { PermissionModeSelector } from "./permission-mode-selector";
import { PlanModeBadge } from "./plan-mode-badge";
import { activeSkillsInText, highlightSkillText } from "@/lib/skill-text";
import type { DroppedFile } from "@/hooks/use-file-drop";
import type { ContextUsageBreakdown } from "@/hooks/context-usage-breakdown";
import type { PermissionMode } from "@/hooks/use-permission-mode";
import type { ReasoningMode } from "@/lib/reasoning-modes";
import "./chat.css";
import "./chat-input-textarea.css";
import "./chat-input-responsive.css";

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
  reasoningMode?: string | null;
  files?: DroppedFile[];
  contextUsed: number;
  contextMax: number;
  contextBreakdown?: ContextUsageBreakdown;
  interactivePending?: boolean;
  permissionMode: PermissionMode;
  planModeEnabled?: boolean;
  onPermissionModeChange: (mode: PermissionMode) => void;
  onPlanModeChange?: (enabled: boolean) => void;
  onSend: (text: string, files?: DroppedFile[], skills?: { name: string; content: string }[]) => void;
  onStop: () => void;
  onFileImport: () => void;
  onModelChange: (model: string, provider: string) => void;
  onReasoningModeChange: (mode: ReasoningMode) => void;
  onRemoveFile?: (index: number) => void;
  onPreviewFile?: (file: DroppedFile) => void;
  onClearFiles?: () => void;
  onBuiltInCommand?: (name: string) => void;
}

export function ChatInput({
  modelName, providerName, isStreaming, reasoningMode, files,
  contextUsed, contextMax, contextBreakdown, interactivePending = false,
  permissionMode, planModeEnabled = false, onPermissionModeChange, onPlanModeChange,
  onSend, onStop, onFileImport, onModelChange, onReasoningModeChange,
  onRemoveFile, onPreviewFile, onClearFiles,
}: ChatInputProps) {
  const { t } = useTranslation();
  const [text, setText] = useState("");
  const { ref, resize } = useAutoResize(200);
  const slash = useSlashCommands();
  const skills = useActiveSkills(slash, text, setText);
  const bubbleRef = useRef<HTMLDivElement>(null);
  const highlightRef = useRef<HTMLDivElement>(null);

  const hasText = text.trim().length > 0;
  const hasFiles = files != null && files.length > 0;
  const hasContent = hasText || hasFiles;
  const visibleSkillNames = activeSkillsInText(text, skills.activeSkills).map((s) => s.name);

  const handleSend = useCallback(() => {
    if (!hasContent || isStreaming || interactivePending) return;
    onSend(text.trim(), hasFiles ? files : undefined, skills.getSkillsPayload());
    setText("");
    skills.clearSkills();
    onClearFiles?.();
    if (ref.current) ref.current.style.height = "auto";
  }, [text, hasContent, hasFiles, files, skills, isStreaming, interactivePending, onSend, onClearFiles, ref]);

  const handleChange = useCallback((value: string) => {
    setText(value);
    resize();
    slash.handleInput(value);
  }, [resize, slash]);

  const handleTextareaScroll = useCallback((e: React.UIEvent<HTMLTextAreaElement>) => {
    if (highlightRef.current) {
      highlightRef.current.scrollTop = e.currentTarget.scrollTop;
    }
  }, []);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    const pressed = eventKey(e);
    if (slash.showDropdown) {
      if (pressed === K_UP) { e.preventDefault(); slash.moveUp(); return; }
      if (pressed === K_DOWN) { e.preventDefault(); slash.moveDown(); return; }
      if (pressed === K_ENTER) {
        e.preventDefault();
        const selected = slash.skills[slash.activeIndex];
        if (selected) void skills.handleSelectSkill(selected);
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
    : hasContent && !interactivePending ? "send" as const
    : "hidden" as const;

  return (
    <div className="chat-input-bubble" data-keyboard-scope="local" ref={bubbleRef}>
      {slash.showDropdown && (
        <SlashAutocomplete
          skills={slash.skills}
          activeIndex={slash.activeIndex}
          onSelect={(s) => void skills.handleSelectSkill(s)}
        />
      )}
      <div className="chat-textarea-shell">
        <div className="chat-textarea-highlight" ref={highlightRef} aria-hidden="true">
          {highlightSkillText(text, visibleSkillNames)}
          {text.endsWith("\n") ? "\n" : null}
        </div>
        <textarea
          ref={ref}
          value={text}
          onChange={(e) => handleChange(e.target.value)}
          onScroll={handleTextareaScroll}
          onKeyDown={handleKeyDown}
          placeholder={interactivePending ? t("interactiveChoice.inputLocked") : t("agentLocal.placeholder")}
          className="chat-textarea"
          disabled={interactivePending}
          rows={2}
        />
      </div>
      {files && files.length > 0 && (
        <div className="chat-file-list">
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
        <ChatPlusMenu
          onFileImport={onFileImport}
          planModeEnabled={planModeEnabled}
          onPlanModeChange={onPlanModeChange ?? (() => {})}
        />
        <ContextProgress used={contextUsed} max={contextMax} breakdown={contextBreakdown} />
        <PermissionModeSelector mode={permissionMode} onChange={onPermissionModeChange} />
        {planModeEnabled && (
          <PlanModeBadge onDisable={() => onPlanModeChange?.(false)} />
        )}
        <div className="chat-input-spacer" />
        <ModelSelector
          selectedModel={modelName}
          selectedProvider={providerName}
          onSelect={onModelChange}
          reasoningMode={reasoningMode}
          onReasoningModeChange={onReasoningModeChange}
          align="right"
        />
        <SendStopButton state={buttonState} onSend={handleSend} onStop={onStop} />
      </div>
    </div>
  );
}

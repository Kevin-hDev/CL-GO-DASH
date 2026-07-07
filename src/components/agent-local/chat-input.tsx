import { useState, useCallback, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { ChatInputActionsRow } from "./chat-input-actions-row";
import { ChatInputEditor } from "./chat-input-editor";
import { InteractiveChoicePanel } from "./interactive-choice-panel";
import { useSlashCommands } from "@/hooks/use-slash-commands";
import { useActiveSkills } from "@/hooks/use-active-skills";
import { SlashAutocomplete } from "./slash-autocomplete";
import { FileThumbnail } from "./file-thumbnail";
import type { DroppedFile } from "@/hooks/use-file-drop";
import type { ContextUsageBreakdown } from "@/hooks/context-usage-breakdown";
import type { PermissionMode } from "@/hooks/use-permission-mode";
import type { ReasoningMode } from "@/lib/reasoning-modes";
import type { AgentInteractiveChoiceRequest } from "@/types/agent";
import type { RetryIndicatorState } from "@/types/agent";
import "./chat.css";
import "./chat-input-textarea.css";
import "./chat-input-responsive.css";

const K_UP = "ArrowUp";
const K_DOWN = "ArrowDown";
const K_ENTER = "Enter";
const K_ESC = "Escape";

interface ChatInputProps {
  modelName: string;
  providerName: string;
  isStreaming: boolean;
  reasoningMode?: string | null;
  files?: DroppedFile[];
  contextUsed: number;
  contextMax: number;
  contextBreakdown?: ContextUsageBreakdown;
  retryIndicator?: RetryIndicatorState | null;
  interactiveRequest?: AgentInteractiveChoiceRequest | null;
  onInteractiveResolved?: () => void;
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
}

export function ChatInput({
  modelName, providerName, isStreaming, reasoningMode, files,
  contextUsed, contextMax, contextBreakdown, retryIndicator,
  interactiveRequest, onInteractiveResolved,
  permissionMode, planModeEnabled = false, onPermissionModeChange, onPlanModeChange,
  onSend, onStop, onFileImport, onModelChange, onReasoningModeChange,
  onRemoveFile, onPreviewFile, onClearFiles,
}: ChatInputProps) {
  const { t } = useTranslation();
  const [text, setText] = useState("");
  const slash = useSlashCommands();
  const skills = useActiveSkills(slash, text, setText);
  const bubbleRef = useRef<HTMLDivElement>(null);

  const interactivePending = !!interactiveRequest;
  const hasText = text.trim().length > 0;
  const hasFiles = files != null && files.length > 0;
  const hasContent = hasText || hasFiles;

  const handleSend = useCallback(() => {
    if (!hasContent || isStreaming || interactivePending) return;
    onSend(text.trim(), hasFiles ? files : undefined, skills.getSkillsPayload());
    setText("");
    skills.clearSkills();
    onClearFiles?.();
  }, [text, hasContent, hasFiles, files, skills, isStreaming, interactivePending, onSend, onClearFiles]);

  const handleChange = useCallback((value: string, cursorPos: number) => {
    setText(value);
    slash.handleInput(value, cursorPos);
  }, [slash]);

  // Shared Enter logic. Bound both as the high-priority keymap (so it wins over
  // defaultKeymap's insertNewlineAndIndent) and inside handleKeyEvent (for the
  // slash-dropdown branch, where we still need the full KeyboardEvent).
  const handleEnter = useCallback((): boolean => {
    if (slash.showDropdown) {
      const selected = slash.skills[slash.activeIndex];
      if (selected) void skills.handleSelectSkill(selected);
      return true;
    }
    handleSend();
    return true;
  }, [handleSend, slash.showDropdown, slash.skills, slash.activeIndex, skills]);

  const handleKeyEvent = useCallback((event: KeyboardEvent): boolean | void => {
    const pressed = event.key;
    if (slash.showDropdown) {
      if (pressed === K_UP) { event.preventDefault(); slash.moveUp(); return true; }
      if (pressed === K_DOWN) { event.preventDefault(); slash.moveDown(); return true; }
      if (pressed === K_ENTER) {
        event.preventDefault();
        return handleEnter();
      }
      if (pressed === K_ESC) { event.preventDefault(); slash.close(); return true; }
    }
    if (pressed === K_ENTER && !event.shiftKey) {
      event.preventDefault();
      return handleEnter();
    }
    if (pressed === K_ESC) {
      event.preventDefault();
      event.stopPropagation();
      if (isStreaming) onStop();
      return true;
    }
  }, [handleEnter, isStreaming, onStop, slash]);

  useEffect(() => {
    if (!isStreaming) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key !== K_ESC) return;
      e.preventDefault();
      onStop();
    };
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
    <div className={`chat-input-bubble${interactivePending ? " chat-input-bubble-interactive" : ""}`} ref={bubbleRef}>
      {interactivePending ? (
        <InteractiveChoicePanel request={interactiveRequest ?? undefined} onResolved={onInteractiveResolved} />
      ) : (
        <>
          {slash.showDropdown && (
            <SlashAutocomplete
              skills={slash.skills}
              activeIndex={slash.activeIndex}
              onSelect={(s) => void skills.handleSelectSkill(s)}
            />
          )}
          <ChatInputEditor
            value={text}
            placeholder={t("agentLocal.placeholder")}
            readOnly={false}
            activeSkills={skills.activeSkills}
            onTextChange={handleChange}
            onEnter={handleEnter}
            onKeyEvent={handleKeyEvent}
          />
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
          <ChatInputActionsRow
            modelName={modelName}
            providerName={providerName}
            reasoningMode={reasoningMode}
            contextUsed={contextUsed}
            contextMax={contextMax}
            contextBreakdown={contextBreakdown}
            permissionMode={permissionMode}
            planModeEnabled={planModeEnabled}
            retryIndicator={retryIndicator}
            buttonState={buttonState}
            onPermissionModeChange={onPermissionModeChange}
            onPlanModeChange={onPlanModeChange}
            onFileImport={onFileImport}
            onModelChange={onModelChange}
            onReasoningModeChange={onReasoningModeChange}
            onSend={handleSend}
            onStop={onStop}
          />
        </>
      )}
    </div>
  );
}

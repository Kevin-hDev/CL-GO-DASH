import { ChatPlusMenu } from "./chat-plus-menu";
import { ContextProgress } from "./context-progress";
import { ModelSelector } from "./model-selector";
import { PermissionModeSelector } from "./permission-mode-selector";
import { PlanModeBadge } from "./plan-mode-badge";
import { RetryIndicator } from "./retry-indicator";
import { SendStopButton } from "./send-stop-button";
import type { ContextUsageBreakdown } from "@/hooks/context-usage-breakdown";
import type { PermissionMode } from "@/hooks/use-permission-mode";
import type { ReasoningMode } from "@/lib/reasoning-modes";
import type { RetryIndicatorState } from "@/types/agent";

type ButtonState = "stop" | "send" | "hidden";

interface ChatInputActionsRowProps {
  modelName: string;
  providerName: string;
  reasoningMode?: string | null;
  contextUsed: number;
  contextMax: number;
  contextBreakdown?: ContextUsageBreakdown;
  permissionMode: PermissionMode;
  planModeEnabled: boolean;
  retryIndicator?: RetryIndicatorState | null;
  buttonState: ButtonState;
  onPermissionModeChange: (mode: PermissionMode) => void;
  onPlanModeChange?: (enabled: boolean) => void;
  onFileImport: () => void;
  onModelChange: (model: string, provider: string) => void;
  onReasoningModeChange: (mode: ReasoningMode) => void;
  onSend: () => void;
  onStop: () => void;
}

export function ChatInputActionsRow({
  modelName,
  providerName,
  reasoningMode,
  contextUsed,
  contextMax,
  contextBreakdown,
  permissionMode,
  planModeEnabled,
  retryIndicator,
  buttonState,
  onPermissionModeChange,
  onPlanModeChange,
  onFileImport,
  onModelChange,
  onReasoningModeChange,
  onSend,
  onStop,
}: ChatInputActionsRowProps) {
  return (
    <div className="chat-input-row3">
      <ChatPlusMenu
        onFileImport={onFileImport}
        planModeEnabled={planModeEnabled}
        onPlanModeChange={onPlanModeChange ?? (() => {})}
      />
      <ContextProgress used={contextUsed} max={contextMax} breakdown={contextBreakdown} />
      <PermissionModeSelector mode={permissionMode} onChange={onPermissionModeChange} />
      <RetryIndicator indicator={retryIndicator} />
      {planModeEnabled && <PlanModeBadge onDisable={() => onPlanModeChange?.(false)} />}
      <div className="chat-input-spacer" />
      <ModelSelector
        selectedModel={modelName}
        selectedProvider={providerName}
        onSelect={onModelChange}
        reasoningMode={reasoningMode}
        onReasoningModeChange={onReasoningModeChange}
        align="right"
      />
      <SendStopButton state={buttonState} onSend={onSend} onStop={onStop} />
    </div>
  );
}

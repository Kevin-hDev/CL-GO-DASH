import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { ChatInput } from "./chat-input";
import { PermissionDialog } from "./permission-dialog";
import { ProjectSelector } from "./project-selector";
import { SubagentAccordion } from "./subagent-accordion";
import type { DroppedFile } from "@/hooks/use-file-drop";
import type { PermissionMode } from "@/hooks/use-permission-mode";
import type { PermissionDecision, PermissionRequest } from "@/hooks/use-permission-requests";
import type { Project, SubagentInfo } from "@/types/agent";

interface ChatInputPanelProps {
  model: string;
  provider: string;
  thinking: boolean;
  files: DroppedFile[];
  contextUsed: number;
  contextMax: number;
  permissionMode: PermissionMode;
  permissionRequest: PermissionRequest | null;
  projects: Project[];
  selectedProjectId: string | null;
  projectLocked: boolean;
  projectHidden: boolean;
  isStreaming: boolean;
  onPermissionRespond: (id: string, decision: PermissionDecision) => void;
  onPermissionModeChange: (mode: PermissionMode) => void;
  onRemoveFile: (index: number) => void;
  onPreviewFile: (file: DroppedFile) => void;
  onSend: (text: string, files?: DroppedFile[], skills?: { name: string; content: string }[]) => void;
  onStop: () => void;
  onClearFiles: () => void;
  onAddFiles: (paths: string[]) => void;
  onModelChange: (model: string, provider: string) => void;
  onToggleThinking: () => void;
  onProjectSelect: (id: string | null) => void;
  onAddProject: () => void;
  activeSubagents?: SubagentInfo[];
  onCancelSubagent?: (sessionId: string) => void;
  onOpenSubagent?: (sessionId: string) => void;
}

export function ChatInputPanel(props: ChatInputPanelProps) {
  return (
    <div className="chat-input-area">
      <div className="chat-input-column">
        {props.activeSubagents && props.activeSubagents.length > 0 && (
          <SubagentAccordion
            subagents={props.activeSubagents}
            onCancel={props.onCancelSubagent ?? (() => {})}
            onOpen={props.onOpenSubagent ?? (() => {})}
          />
        )}
        {props.permissionRequest && (
          <PermissionDialog
            request={props.permissionRequest}
            onDecide={props.onPermissionRespond}
          />
        )}
        <ChatInput
          modelName={props.model}
          providerName={props.provider}
          isStreaming={props.isStreaming}
          thinkingEnabled={props.thinking}
          files={props.files}
          contextUsed={props.contextUsed}
          contextMax={props.contextMax}
          permissionMode={props.permissionMode}
          onPermissionModeChange={props.onPermissionModeChange}
          onRemoveFile={props.onRemoveFile}
          onPreviewFile={props.onPreviewFile}
          onSend={props.onSend}
          onStop={props.onStop}
          onClearFiles={props.onClearFiles}
          onFileImport={() => void importFiles(props.onAddFiles)}
          onModelChange={props.onModelChange}
          onToggleThinking={props.onToggleThinking}
        />
        <ProjectSelector
          projects={props.projects}
          selectedProjectId={props.selectedProjectId}
          locked={props.projectLocked}
          hidden={props.projectHidden}
          onSelect={props.onProjectSelect}
          onAddProject={props.onAddProject}
        />
      </div>
    </div>
  );
}

async function importFiles(onAddFiles: (paths: string[]) => void) {
  const result = await openFileDialog({ multiple: true });
  if (!result) return;
  const raw = Array.isArray(result) ? result : [result];
  onAddFiles(raw.map((path) => String(path)));
}

import { Group, Panel, Separator } from "react-resizable-panels";
import { ChatView } from "./chat-view";
import { FilePreviewPanel } from "@/components/file-preview/file-preview-panel";
import type { useFilePreview } from "@/hooks/use-file-preview";
import type { DroppedFile } from "@/hooks/use-file-drop";
import type { useTerminal } from "@/hooks/use-terminal";
import type { Project } from "@/types/agent";
import type { FileOperation } from "@/types/file-preview";

interface AgentChatDetailProps {
  sessionId: string;
  model: string;
  provider: string;
  projects: Project[];
  activeProjectPath?: string;
  pendingMessage?: string | null;
  pendingWorkingDir?: string;
  pendingSkills?: { name: string; content: string }[];
  pendingFiles?: DroppedFile[];
  thinking: boolean;
  terminal: ReturnType<typeof useTerminal>;
  filePreview: ReturnType<typeof useFilePreview>;
  fileOperations: FileOperation[];
  onAddProject: (path: string) => Promise<Project>;
  onSessionsRefresh: () => void;
  onUpdateModel: (id: string, model: string, provider: string) => void;
  onNewSession: (model: string, provider: string) => void;
  onAutoRename: (id: string, name: string) => void;
  onToggleThinking: () => void;
  onInitialMessageSent: () => void;
  onFileOperationsChange: (operations: FileOperation[]) => void;
}

export function AgentChatDetail(props: AgentChatDetailProps) {
  const chatView = (
    <ChatView
      sessionId={props.sessionId}
      model={props.model}
      provider={props.provider}
      projects={props.projects}
      onAddProject={props.onAddProject}
      onSessionsRefresh={props.onSessionsRefresh}
      onApplySwitch={(model, provider) => props.onUpdateModel(props.sessionId, model, provider)}
      onNewSession={props.onNewSession}
      onAutoRename={props.onAutoRename}
      initialMessage={props.pendingMessage ?? undefined}
      initialWorkingDir={props.pendingWorkingDir}
      initialSkills={props.pendingSkills}
      initialFiles={props.pendingFiles}
      thinking={props.thinking}
      onToggleThinking={props.onToggleThinking}
      onInitialMessageSent={props.onInitialMessageSent}
      terminalState={props.terminal}
      onFileOperationsChange={props.onFileOperationsChange}
      onFilePreviewPath={props.filePreview.openPath}
    />
  );

  if (!props.filePreview.open) {
    return (
      <div className="agent-detail-with-preview">
        <div style={{ flex: 1, minWidth: 0, minHeight: 0, overflow: "hidden" }}>
          {chatView}
        </div>
      </div>
    );
  }

  return (
    <div className="agent-detail-with-preview">
      <Group id="file-preview-panel-sizes" orientation="horizontal">
        <Panel id="chat" defaultSize={65} minSize={30} style={{ overflow: "hidden" }}>
          {chatView}
        </Panel>
        <Separator className="fp-panel-handle" />
        <Panel id="preview" defaultSize={35} minSize={20} maxSize={70} style={{ minWidth: 0 }}>
          <FilePreviewPanel
            open={props.filePreview.open}
            fullscreen={props.filePreview.fullscreen}
            operations={props.fileOperations}
            tabs={props.filePreview.tabs}
            activeTab={props.filePreview.activeTab}
            baseDir={props.activeProjectPath}
            onClose={() => props.filePreview.setOpen(false)}
            onFullscreenChange={props.filePreview.setFullscreen}
            onActiveTabChange={props.filePreview.setActiveTab}
            onOpenOperation={props.filePreview.openOperation}
            onCloseTab={props.filePreview.closeTab}
          />
        </Panel>
      </Group>
    </div>
  );
}

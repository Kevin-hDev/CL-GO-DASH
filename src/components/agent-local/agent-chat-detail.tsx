import { ChatView } from "./chat-view";
import { FilePreviewPanel } from "@/components/file-preview/file-preview-panel";
import { FileTreePanel } from "@/components/file-tree/file-tree-panel";
import type { useFilePreview } from "@/hooks/use-file-preview";
import type { useFileTree } from "@/hooks/use-file-tree";
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
  fileTree: ReturnType<typeof useFileTree>;
  parentSessionId?: string;
  onOpenSubagent?: (sessionId: string) => void;
  onGoToParent?: () => void;
}

export function AgentChatDetail(props: AgentChatDetailProps) {
  return (
    <div className="agent-detail-with-preview">
      {props.parentSessionId && (
        <button
          className="sa-parent-btn"
          onClick={props.onGoToParent}
          type="button"
        >
          ← Chat parent
        </button>
      )}
      <div style={{ flex: 1, minWidth: 0, minHeight: 0, overflow: "hidden" }}>
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
          onOpenSubagent={props.onOpenSubagent}
          isSubagent={!!props.parentSessionId}
        />
      </div>
      <div className={`panels-wrapper ${props.filePreview.fullscreen ? "panels-fullscreen" : ""}`}>
        <FilePreviewPanel
          open={props.filePreview.open}
          fullscreen={props.filePreview.fullscreen}
          width={props.filePreview.width}
          resizing={props.filePreview.resizing}
          operations={props.fileOperations}
          tabs={props.filePreview.tabs}
          activeTab={props.filePreview.activeTab}
          baseDir={props.activeProjectPath}
          onClose={props.filePreview.closePanel}
          onFullscreenChange={props.filePreview.setFullscreen}
          onActiveTabChange={props.filePreview.setActiveTab}
          onOpenOperation={props.filePreview.openOperation}
          onCloseTab={props.filePreview.closeTab}
          onResizeStart={props.filePreview.startResize}
          hasProject={props.fileTree.hasProject}
          treeOpen={props.fileTree.open}
          onToggleTree={props.fileTree.toggleOpen}
        />
        <FileTreePanel
          tree={props.fileTree}
          onFileSelect={props.filePreview.openPath}
          activePath={props.filePreview.tabs.find((tab) => tab.id === props.filePreview.activeTab)?.path ?? null}
        />
      </div>
    </div>
  );
}

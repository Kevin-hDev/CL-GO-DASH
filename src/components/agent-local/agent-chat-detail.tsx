import type { CSSProperties } from "react";
import { ChatView } from "./chat-view";
import { FilePreviewPanel } from "@/components/file-preview/file-preview-panel";
import { FileTreePanel } from "@/components/file-tree/file-tree-panel";
import type { useFilePreview } from "@/hooks/use-file-preview";
import type { useFileTree } from "@/hooks/use-file-tree";
import { useAgentPanelLayout } from "@/hooks/use-agent-panel-layout";
import type { DroppedFile } from "@/hooks/use-file-drop";
import type { useTerminal } from "@/hooks/use-terminal";
import type { Project } from "@/types/agent";
import type { FileOperation } from "@/types/file-preview";
import type { PanelMode } from "@/hooks/use-forecast-panel";
import type { ReasoningMode } from "@/lib/reasoning-modes";

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
  reasoningMode?: string | null;
  terminal: ReturnType<typeof useTerminal>;
  filePreview: ReturnType<typeof useFilePreview>;
  fullscreenSwitching: boolean;
  fileOperations: FileOperation[];
  onAddProject: (path: string) => Promise<Project>;
  onSessionsRefresh: () => void;
  onUpdateModel: (id: string, model: string, provider: string) => void;
  onNewSession: (model: string, provider: string) => void;
  onNewSessionInProject: (model: string, provider: string, projectId: string) => void;
  onAutoRename: (id: string, name: string) => void;
  onReasoningModeChange: (mode: ReasoningMode) => void;
  onInitialMessageSent: () => void;
  onFileOperationsChange: (operations: FileOperation[]) => void;
  onPreviewFullscreenChange: (fullscreen: boolean) => void;
  fileTree: ReturnType<typeof useFileTree>;
  parentSessionId?: string;
  onOpenSubagent?: (sessionId: string) => void;
  onGoToParent?: () => void;
  panelMode?: PanelMode;
  forecastContent?: React.ReactNode;
}

export function AgentChatDetail(props: AgentChatDetailProps) {
  const previewDesiredWidth = props.filePreview.width + props.filePreview.extraWidth;
  const { containerRef, layout } = useAgentPanelLayout({
    previewOpen: props.filePreview.open,
    previewFullscreen: props.filePreview.fullscreen,
    previewDesiredWidth,
    fileTreeOpen: props.fileTree.open,
    fileTreeDesiredWidth: props.fileTree.width,
  });
  const layoutStyle = {
    "--agent-chat-min-width": `${layout.chatMinWidth}px`,
    "--ft-active-width": `${props.fileTree.open ? layout.fileTreeWidth : 0}px`,
  } as CSSProperties;

  return (
    <div className="agent-detail-with-preview" ref={containerRef} style={layoutStyle}>
      {props.parentSessionId && (
        <button
          className="sa-parent-btn"
          onClick={props.onGoToParent}
          type="button"
        >
          ← Chat parent
        </button>
      )}
      <div className={`agent-detail-chat ${props.filePreview.fullscreen ? "agent-detail-chat-fs" : ""} ${props.fullscreenSwitching ? "agent-detail-chat-instant" : ""}`}>
        <ChatView
          sessionId={props.sessionId}
          model={props.model}
          provider={props.provider}
          projects={props.projects}
          onAddProject={props.onAddProject}
          onSessionsRefresh={props.onSessionsRefresh}
          onApplySwitch={(model, provider) => props.onUpdateModel(props.sessionId, model, provider)}
          onNewSession={props.onNewSession}
          onNewSessionInProject={props.onNewSessionInProject}
          onAutoRename={props.onAutoRename}
          initialMessage={props.pendingMessage ?? undefined}
          initialWorkingDir={props.pendingWorkingDir}
          initialSkills={props.pendingSkills}
          initialFiles={props.pendingFiles}
          reasoningMode={props.reasoningMode}
          onReasoningModeChange={props.onReasoningModeChange}
          onInitialMessageSent={props.onInitialMessageSent}
          terminalState={props.terminal}
          onFileOperationsChange={props.onFileOperationsChange}
          onFilePreviewPath={props.filePreview.openPath}
          onOpenSubagent={props.onOpenSubagent}
          isSubagent={!!props.parentSessionId}
        />
      </div>
      <FilePreviewPanel
        open={props.filePreview.open}
        fullscreen={props.filePreview.fullscreen}
        width={props.filePreview.width}
        displayWidth={layout.previewWidth}
        extraWidth={props.filePreview.extraWidth}
        fullscreenWidth={props.filePreview.fullscreenWidth}
        fullscreenSwitching={props.fullscreenSwitching}
        resizing={props.filePreview.resizing}
        operations={props.fileOperations}
        tabs={props.filePreview.tabs}
        activeTab={props.filePreview.activeTab}
        baseDir={props.activeProjectPath}
        onClose={props.filePreview.closePanel}
        onFullscreenChange={props.onPreviewFullscreenChange}
        onActiveTabChange={props.filePreview.setActiveTab}
        onOpenOperation={props.filePreview.openOperation}
        onCloseTab={props.filePreview.closeTab}
        onResizeStart={props.filePreview.startResize}
        hasProject={props.fileTree.hasProject}
        treeOpen={props.fileTree.open}
        onToggleTree={props.fileTree.toggleOpen}
        panelMode={props.panelMode}
        forecastContent={props.forecastContent}
      />
      <FileTreePanel
        tree={props.fileTree}
        displayWidth={layout.fileTreeWidth}
        onFileSelect={props.filePreview.openPath}
        activePath={props.filePreview.tabs.find((tab) => tab.id === props.filePreview.activeTab)?.path ?? null}
      />
    </div>
  );
}

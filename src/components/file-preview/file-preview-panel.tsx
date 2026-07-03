import { useTranslation } from "react-i18next";
import { Maximize2, Minimize2, FolderTree } from "@/components/ui/lucide-icons";
import { openPreviewFile, openPreviewWithEditor } from "@/services/file-preview";
import type { FileOperation, FilePreviewActiveTab, FilePreviewListMode } from "@/types/file-preview";
import type { PanelMode } from "@/hooks/use-forecast-panel";
import { shouldWrapFile } from "@/lib/code-language";
import { FilePreviewBreadcrumb } from "./file-preview-breadcrumb";
import { FilePreviewContent } from "./file-preview-content";
import { FilePreviewPlan } from "./file-preview-plan";
import { FilePreviewSummary } from "./file-preview-summary";
import { FilePreviewTabs } from "./file-preview-tabs";
import "./file-preview-panel.css";
import "./file-preview-tabs.css";
import "./file-preview-highlight.css";

interface FilePreviewPanelProps {
  open: boolean;
  fullscreen: boolean;
  width: number;
  displayWidth?: number;
  extraWidth?: number;
  fullscreenWidth: number;
  fullscreenSwitching: boolean;
  resizing: boolean;
  allOperations: FileOperation[];
  latestOperations: FileOperation[];
  tabs: FileOperation[];
  activeTab: FilePreviewActiveTab;
  listMode: FilePreviewListMode;
  baseDir?: string;
  onClose: () => void;
  onFullscreenChange: (fullscreen: boolean) => void;
  onActiveTabChange: (tab: FilePreviewActiveTab) => void;
  onListModeChange: (mode: FilePreviewListMode) => void;
  onOpenOperation: (operation: FileOperation) => void;
  onOpenFilePath: (path: string) => void;
  onCloseTab: (id: string) => void;
  onResizeStart: (event: React.PointerEvent) => void;
  hasProject?: boolean;
  treeOpen?: boolean;
  onToggleTree?: () => void;
  panelMode?: PanelMode;
  forecastContent?: React.ReactNode;
}

export function FilePreviewPanel(props: FilePreviewPanelProps) {
  const { t } = useTranslation();
  const activeOperation = props.tabs.find((tab) => tab.id === props.activeTab);
  const summaryOperations = props.listMode === "latest" ? props.latestOperations : props.allOperations;

  const openDefault = (operation: FileOperation) => {
    openPreviewFile(operation.path, props.baseDir).catch(() => {});
  };

  const openWith = (operation: FileOperation, editorPath: string) => {
    openPreviewWithEditor(operation.path, editorPath, props.baseDir).catch(() => {});
  };

  return (
    <aside
      className={`fp-panel ${props.open ? "open" : ""} ${props.fullscreen ? "fullscreen" : ""} ${props.fullscreenSwitching ? "fullscreen-switching" : ""} ${props.resizing ? "resizing" : ""}`}
      data-nav-zone="filePreview"
      style={{
        "--fp-width": `${props.displayWidth ?? props.width + (props.extraWidth ?? 0)}px`,
        "--fp-full-width": `${props.fullscreenWidth}px`,
      } as React.CSSProperties}
      aria-hidden={!props.open}
    >
      <div className="fp-resize" onPointerDown={props.onResizeStart} />
      <div className={`fp-slide-wrapper ${props.panelMode === "forecast" ? "fp-slide-forecast" : "fp-slide-preview"}`}>
        <div className="fp-slide-child">
          <div className="fp-head">
            <FilePreviewTabs
              tabs={props.tabs}
              activeTab={props.activeTab}
              listMode={props.listMode}
              baseDir={props.baseDir}
              onSelect={props.onActiveTabChange}
              onListModeChange={props.onListModeChange}
              onClose={props.onCloseTab}
              onOpenDefault={openDefault}
              onOpenWith={openWith}
            />
            {props.hasProject && (
              <button
                className={`fp-icon-btn ${props.treeOpen ? "fp-icon-btn-active" : ""}`}
                onClick={props.onToggleTree}
                title={t("fileTree.toggleTree")}
              >
                <FolderTree size="var(--icon-md)" />
              </button>
            )}
            <button
              className="fp-icon-btn"
              onClick={() => props.onFullscreenChange(!props.fullscreen)}
              title={props.fullscreen ? t("filePreview.reduce") : t("filePreview.fullscreen")}
            >
              {props.fullscreen ? <Minimize2 size="var(--icon-md)" /> : <Maximize2 size="var(--icon-md)" />}
            </button>
          </div>
          <div className="fp-body">
            {props.activeTab === "summary" || !activeOperation ? (
              <div className="fp-summary-scroll">
                <FilePreviewSummary
                  operations={summaryOperations}
                  baseDir={props.baseDir}
                  onOpen={props.onOpenOperation}
                  onOpenFile={(operation) => props.onOpenFilePath(operation.path)}
                />
              </div>
            ) : activeOperation.kind === "plan" ? (
              <FilePreviewPlan operation={activeOperation} baseDir={props.baseDir} />
            ) : (
              <>
                <FilePreviewBreadcrumb
                  operation={activeOperation}
                  baseDir={props.baseDir}
                />
                <div className={`fp-code-scroll ${shouldWrapFile(activeOperation.path) ? "" : "fp-nowrap"}`}>
                  <FilePreviewContent key={activeOperation.id} operation={activeOperation} baseDir={props.baseDir} />
                </div>
              </>
            )}
          </div>
        </div>
        <div className="fp-slide-child">
          {props.forecastContent}
        </div>
      </div>
    </aside>
  );
}

import { useTranslation } from "react-i18next";
import { Maximize2, Minimize2, FolderTree } from "lucide-react";
import { openPreviewFile, openPreviewWithEditor } from "@/services/file-preview";
import type { FileOperation, FilePreviewActiveTab } from "@/types/file-preview";
import type { PanelMode } from "@/hooks/use-forecast-panel";
import { shouldWrapFile } from "@/lib/code-language";
import { FilePreviewBreadcrumb } from "./file-preview-breadcrumb";
import { FilePreviewContent } from "./file-preview-content";
import { FilePreviewSummary } from "./file-preview-summary";
import { FilePreviewTabs } from "./file-preview-tabs";
import "./file-preview-panel.css";
import "./file-preview-tabs.css";
import "./file-preview-highlight.css";

interface FilePreviewPanelProps {
  open: boolean;
  fullscreen: boolean;
  width: number;
  extraWidth?: number;
  fullscreenWidth: number;
  fullscreenSwitching: boolean;
  resizing: boolean;
  operations: FileOperation[];
  tabs: FileOperation[];
  activeTab: FilePreviewActiveTab;
  baseDir?: string;
  onClose: () => void;
  onFullscreenChange: (fullscreen: boolean) => void;
  onActiveTabChange: (tab: FilePreviewActiveTab) => void;
  onOpenOperation: (operation: FileOperation) => void;
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

  const openDefault = (operation: FileOperation) => {
    openPreviewFile(operation.path, props.baseDir).catch(() => {});
  };

  const openWith = (operation: FileOperation, editorPath: string) => {
    openPreviewWithEditor(operation.path, editorPath, props.baseDir).catch(() => {});
  };

  return (
    <aside
      className={`fp-panel ${props.open ? "open" : ""} ${props.fullscreen ? "fullscreen" : ""} ${props.fullscreenSwitching ? "fullscreen-switching" : ""} ${props.resizing ? "resizing" : ""}`}
      style={{
        "--fp-width": `${props.width + (props.extraWidth ?? 0)}px`,
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
              baseDir={props.baseDir}
              onSelect={props.onActiveTabChange}
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
                <FolderTree size={16} />
              </button>
            )}
            <button
              className="fp-icon-btn"
              onClick={() => props.onFullscreenChange(!props.fullscreen)}
              title={props.fullscreen ? t("filePreview.reduce") : t("filePreview.fullscreen")}
            >
              {props.fullscreen ? <Minimize2 size={16} /> : <Maximize2 size={16} />}
            </button>
          </div>
          <div className="fp-body">
            {props.activeTab === "summary" || !activeOperation ? (
              <div className="fp-summary-scroll">
                <FilePreviewSummary
                  operations={props.operations}
                  baseDir={props.baseDir}
                  onOpen={props.onOpenOperation}
                />
              </div>
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

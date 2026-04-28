import { useEffect, useState } from "react";
import { Maximize2, Minimize2 } from "lucide-react";
import { detectPreviewEditors, openPreviewFile, openPreviewWithEditor } from "@/services/file-preview";
import type { FileOperation, FilePreviewActiveTab, PreviewEditor } from "@/types/file-preview";
import { FilePreviewBreadcrumb } from "./file-preview-breadcrumb";
import { FilePreviewContent } from "./file-preview-content";
import { FilePreviewSummary } from "./file-preview-summary";
import { FilePreviewTabs } from "./file-preview-tabs";
import "./file-preview-panel.css";
import "./file-preview-tabs.css";
import "./file-preview-code.css";

interface FilePreviewPanelProps {
  open: boolean;
  fullscreen: boolean;
  operations: FileOperation[];
  tabs: FileOperation[];
  activeTab: FilePreviewActiveTab;
  baseDir?: string;
  onClose: () => void;
  onFullscreenChange: (fullscreen: boolean) => void;
  onActiveTabChange: (tab: FilePreviewActiveTab) => void;
  onOpenOperation: (operation: FileOperation) => void;
  onCloseTab: (id: string) => void;
}

export function FilePreviewPanel(props: FilePreviewPanelProps) {
  const [editors, setEditors] = useState<PreviewEditor[]>([]);
  const activeOperation = props.tabs.find((tab) => tab.id === props.activeTab);

  useEffect(() => {
    if (!props.open) return;
    detectPreviewEditors().then(setEditors).catch(() => setEditors([]));
  }, [props.open]);

  const openDefault = (operation: FileOperation) => {
    openPreviewFile(operation.path, props.baseDir).catch(() => {});
  };

  const openWith = (operation: FileOperation, editorId: string) => {
    openPreviewWithEditor(operation.path, editorId, props.baseDir).catch(() => {});
  };

  return (
    <aside
      className={`fp-panel ${props.fullscreen ? "fullscreen" : ""}`}
      aria-hidden={!props.open}
    >
      <div className="fp-head">
        <FilePreviewTabs
          tabs={props.tabs}
          activeTab={props.activeTab}
          editors={editors}
          onSelect={props.onActiveTabChange}
          onClose={props.onCloseTab}
          onOpenDefault={openDefault}
          onOpenWith={openWith}
        />
        <button
          className="fp-icon-btn"
          onClick={() => props.onFullscreenChange(!props.fullscreen)}
          title={props.fullscreen ? "Réduire" : "Plein écran"}
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
            {editors.length === 0 && <div className="fp-editor-note">Aucun éditeur externe détecté.</div>}
            <div className="fp-code-scroll">
              <FilePreviewContent operation={activeOperation} baseDir={props.baseDir} />
            </div>
          </>
        )}
      </div>
    </aside>
  );
}

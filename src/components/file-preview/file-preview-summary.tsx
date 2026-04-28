import { useTranslation } from "react-i18next";
import { shortPath } from "@/lib/file-preview-utils";
import type { FileOperation } from "@/types/file-preview";
import { FileIcon } from "./file-icon";
import { FilePreviewStats } from "./file-preview-stats";
import "./file-preview-summary.css";

interface FilePreviewSummaryProps {
  operations: FileOperation[];
  baseDir?: string;
  onOpen: (operation: FileOperation) => void;
}

export function FilePreviewSummary({
  operations,
  baseDir,
  onOpen,
}: FilePreviewSummaryProps) {
  const { t } = useTranslation();

  if (operations.length === 0) {
    return (
      <div className="fp-empty">
        {t("filePreview.noFiles")}
      </div>
    );
  }

  return (
    <div className="fp-summary">
      <div className="fp-summary-title">
        {t("filePreview.filesModified")} <span>{operations.length}</span>
      </div>
      {operations.map((operation) => (
        <button
          key={operation.id}
          className="fp-summary-row"
          onClick={() => onOpen(operation)}
        >
          <FileIcon name={operation.name} />
          <span className="fp-summary-main">
            <span className="fp-summary-name">{operation.name}</span>
            <span className="fp-summary-path">{shortPath(operation.path, baseDir)}</span>
          </span>
          <FilePreviewStats operation={operation} />
        </button>
      ))}
    </div>
  );
}

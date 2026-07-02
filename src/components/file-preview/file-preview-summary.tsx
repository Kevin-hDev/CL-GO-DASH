import { useTranslation } from "react-i18next";
import { ArrowSquareOut } from "@/components/ui/icons";
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
      {operations.map((operation) => {
        const path = splitDisplayPath(shortPath(operation.path, baseDir), operation.name);
        return (
          <button
            key={operation.id}
            className="fp-summary-row"
            onClick={() => onOpen(operation)}
          >
            <FileIcon name={operation.name} size={18} />
            <span className="fp-summary-main" title={path.full}>
              {path.prefix && <span className="fp-summary-path">{path.prefix}</span>}
              <span className="fp-summary-name">{operation.name}</span>
            </span>
            <FilePreviewStats operation={operation} showZero />
            <ArrowSquareOut className="fp-summary-open" size="var(--icon-xs)" aria-hidden="true" />
          </button>
        );
      })}
    </div>
  );
}

function splitDisplayPath(path: string, name: string) {
  const normalizedPath = path.replaceAll("\\", "/");
  const normalizedName = name.replaceAll("\\", "/");
  const prefix = normalizedPath.endsWith(normalizedName)
    ? normalizedPath.slice(0, normalizedPath.length - normalizedName.length)
    : "";
  return { full: path, prefix };
}

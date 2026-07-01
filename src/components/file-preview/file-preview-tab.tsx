import { List, X } from "@/components/ui/lucide-icons";
import { FileIcon } from "./file-icon";
import type { FileOperation } from "@/types/file-preview";

interface FilePreviewTabProps {
  operation?: FileOperation;
  active: boolean;
  label: string;
  summary?: boolean;
  onSelect: () => void;
  onClose?: () => void;
  onContextMenu?: (event: React.MouseEvent) => void;
}

export function FilePreviewTab({
  operation,
  active,
  label,
  summary = false,
  onSelect,
  onClose,
  onContextMenu,
}: FilePreviewTabProps) {
  return (
    <button
      className={`fp-tab ${active ? "active" : ""}`}
      onClick={onSelect}
      onContextMenu={summary ? undefined : onContextMenu}
    >
      <span className="fp-tab-icon">
        <span className="fp-tab-file-icon">
          {summary ? <List size="var(--icon-15)" /> : <FileIcon name={operation?.name ?? label} size="var(--icon-15)" />}
        </span>
        {!summary && <X className="fp-tab-close" size="var(--icon-15)" onClick={(event) => {
          event.stopPropagation();
          onClose?.();
        }} />}
      </span>
      <span className="fp-tab-label">{label}</span>
    </button>
  );
}

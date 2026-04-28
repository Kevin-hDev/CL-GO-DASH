import type { FileOperation } from "@/types/file-preview";

export function FilePreviewStats({ operation }: { operation: FileOperation }) {
  return (
    <span className="fp-stats">
      {operation.additions > 0 && <span className="fp-add">+{operation.additions}</span>}
      {operation.deletions > 0 && <span className="fp-del">-{operation.deletions}</span>}
    </span>
  );
}

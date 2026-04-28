import type { FileOperation } from "@/types/file-preview";
import { FilePreviewHighlight } from "./file-preview-highlight";

export function FilePreviewDiff({
  operation,
}: {
  operation: FileOperation;
  currentContent: string;
}) {
  const oldText = operation.oldText ?? "";
  const newText = operation.newText ?? "";

  return (
    <div className="fp-diff">
      {oldText && <FilePreviewHighlight code={oldText} path={operation.path} mode="del" />}
      {newText && <FilePreviewHighlight code={newText} path={operation.path} mode="add" />}
    </div>
  );
}

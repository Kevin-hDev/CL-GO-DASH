import { useMemo } from "react";
import { DiffView, DiffModeEnum } from "@git-diff-view/react";
import "@git-diff-view/react/styles/diff-view-pure.css";
import type { FileOperation } from "@/types/file-preview";
import { generateHunks, buildOldContent } from "./file-preview-diff-utils";

export function FilePreviewDiff({
  operation,
  currentContent,
}: {
  operation: FileOperation;
  currentContent: string;
}) {
  const diffData = useMemo(() => {
    const newContent = currentContent;
    const oldContent = buildOldContent(currentContent, operation.oldText, operation.newText);
    const hunks = generateHunks(oldContent, newContent);
    const ext = operation.path.split(".").pop() ?? "";
    return {
      oldFile: { fileName: operation.name, fileLang: ext, content: oldContent },
      newFile: { fileName: operation.name, fileLang: ext, content: newContent },
      hunks,
    };
  }, [currentContent, operation]);

  return (
    <div className="fp-diff-view">
      <DiffView
        data={diffData}
        diffViewMode={DiffModeEnum.Unified}
        diffViewWrap
        diffViewFontSize={13}
        diffViewTheme="dark"
      />
    </div>
  );
}

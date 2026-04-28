import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { readFilePreview } from "@/services/file-preview";
import type { FileOperation } from "@/types/file-preview";
import { FilePreviewDiff } from "./file-preview-diff";
import { FilePreviewHighlight } from "./file-preview-highlight";

interface FilePreviewContentProps {
  operation: FileOperation;
  baseDir?: string;
}

export function FilePreviewContent({ operation, baseDir }: FilePreviewContentProps) {
  const { t } = useTranslation();
  const [state, setState] = useState<{ loading: boolean; content: string; error: boolean }>({
    loading: true,
    content: "",
    error: false,
  });

  useEffect(() => {
    let alive = true;
    setState({ loading: true, content: "", error: false });
    readFilePreview(operation.path, baseDir)
      .then((content) => {
        if (alive) setState({ loading: false, content, error: false });
      })
      .catch(() => {
        if (alive) setState({ loading: false, content: "", error: true });
      });
    return () => { alive = false; };
  }, [operation.path, baseDir]);

  if (state.loading) {
    return <div className="fp-empty">{t("filePreview.loading")}</div>;
  }
  if (state.error) {
    return <div className="fp-empty">{t("filePreview.fileNotFound")}</div>;
  }
  if (operation.type === "edit") {
    return <FilePreviewDiff operation={operation} currentContent={state.content} />;
  }
  return <FilePreviewHighlight code={state.content} path={operation.path} mode="add" />;
}

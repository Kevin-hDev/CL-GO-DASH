import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { readFilePreview } from "@/services/file-preview";
import type { FileOperation } from "@/types/file-preview";
import { FilePreviewDiff } from "./file-preview-diff";
import "@/components/agent-local/tool-previews.css";

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

  const lines = state.content.split("\n");
  return (
    <div className="tp-wrapper" style={{ margin: 0, border: "none", borderRadius: 0 }}>
      {lines.map((line, i) => (
        <div key={i} className="tp-line tp-line-ok">
          <span className="tp-num">{i + 1}</span>
          <span className="tp-prefix tp-prefix-ok">+</span>
          <span className="tp-code tp-code-ok">{line}</span>
        </div>
      ))}
    </div>
  );
}

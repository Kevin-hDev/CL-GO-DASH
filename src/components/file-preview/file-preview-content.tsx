import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { readFilePreview } from "@/services/file-preview";
import { highlightLines } from "@/lib/highlight";
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

  const highlighted = useMemo(
    () => state.content ? highlightLines(state.content, operation.path) : [],
    [state.content, operation.path],
  );

  if (state.loading) {
    return <div className="fp-empty">{t("filePreview.loading")}</div>;
  }
  if (state.error) {
    return <div className="fp-empty">{t("filePreview.fileNotFound")}</div>;
  }
  if (operation.type === "edit") {
    return <FilePreviewDiff operation={operation} currentContent={state.content} />;
  }

  return (
    <div className="tp-wrapper" style={{ margin: 0, border: "none", borderRadius: 0 }}>
      {highlighted.map((html, i) => (
        <div key={i} className="tp-line tp-line-ok">
          <span className="tp-num">{i + 1}</span>
          <span className="tp-prefix tp-prefix-ok">+</span>
          <span className="tp-code tp-code-ok" dangerouslySetInnerHTML={{ __html: html || " " }} />
        </div>
      ))}
    </div>
  );
}

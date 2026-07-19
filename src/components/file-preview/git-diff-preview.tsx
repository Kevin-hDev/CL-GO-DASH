import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { highlightLines } from "@/lib/highlight";
import { shouldWrapFile } from "@/lib/code-language";
import { readGitDiffPreview } from "@/services/file-preview";
import type { GitDiffHunk, GitDiffPreview as GitDiffData, GitDiffPreviewSource } from "@/types/file-preview";
import "./git-diff-preview.css";

interface GitDiffPreviewProps {
  source: GitDiffPreviewSource;
  path: string;
  baseDir?: string;
}

interface HighlightedHunk extends GitDiffHunk {
  highlighted: string[];
}

export function GitDiffPreview({ source, path, baseDir }: GitDiffPreviewProps) {
  const { t } = useTranslation();
  const [state, setState] = useState<{
    loading: boolean;
    data?: GitDiffData;
    error: boolean;
  }>({ loading: true, error: false });

  useEffect(() => {
    let alive = true;
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    setState({ loading: true, error: false });
    readGitDiffPreview(source, baseDir)
      .then((data) => {
        if (alive) setState({ loading: false, data, error: false });
      })
      .catch(() => {
        if (alive) setState({ loading: false, error: true });
      });
    return () => { alive = false; };
  }, [source, baseDir]);

  const hunks = useMemo<HighlightedHunk[]>(() => (
    state.data?.hunks.map((hunk) => ({
      ...hunk,
      highlighted: highlightLines(hunk.lines.map((line) => line.content).join("\n"), path),
    })) ?? []
  ), [state.data, path]);

  if (state.loading) return <div className="fp-empty">{t("filePreview.loading")}</div>;
  const isRename = Boolean(source.previousPath);
  const wrap = shouldWrapFile(path);
  if (state.error || state.data?.binary || (hunks.length === 0 && !isRename)) {
    return <div className="fp-empty">{t("filePreview.diffUnavailable")}</div>;
  }

  const content = (
    <>
      {source.previousPath && (
        <div className="gdp-rename">
          <span>{source.previousPath}</span>
          <span aria-hidden="true">→</span>
          <span>{source.filePath}</span>
        </div>
      )}
      {hunks.map((hunk, hunkIndex) => (
        <div className="gdp-hunk" key={`${hunk.old_start}:${hunk.new_start}:${hunkIndex}`}>
          <div className="gdp-hunk-header">
            {`@@ -${formatRange(hunk.old_start, hunk.old_lines)} +${formatRange(hunk.new_start, hunk.new_lines)} @@`}
          </div>
          {hunk.lines.map((line, lineIndex) => {
            const mode = line.kind === "added" ? "ok" : line.kind === "deleted" ? "error" : "context";
            const prefix = line.kind === "added" ? "+" : line.kind === "deleted" ? "-" : " ";
            return (
              <div className={`tp-line tp-line-${mode}`} key={lineIndex}>
                <span className="gdp-line-number">{line.old_line ?? ""}</span>
                <span className="gdp-line-number">{line.new_line ?? ""}</span>
                <span className={`tp-prefix tp-prefix-${mode}`}>{prefix}</span>
                <span
                  className={`tp-code tp-code-${mode}`}
                  dangerouslySetInnerHTML={{ __html: hunk.highlighted[lineIndex] || " " }}
                />
              </div>
            );
          })}
        </div>
      ))}
      {state.data?.truncated && <div className="gdp-note">{t("filePreview.diffTruncated")}</div>}
    </>
  );

  return (
    <div className={`tp-wrapper gdp-wrapper ${wrap ? "" : "tp-nowrap"}`}>
      {wrap ? content : <div className="tp-inner">{content}</div>}
    </div>
  );
}

function formatRange(start: number, count: number): string {
  return count === 1 ? String(start) : `${start},${count}`;
}

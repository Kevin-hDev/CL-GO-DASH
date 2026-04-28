import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { codeToHtml } from "shiki";
import { languageFromPath } from "@/lib/code-language";

interface FilePreviewHighlightProps {
  code: string;
  path: string;
  mode?: "normal" | "add" | "del";
}

function useCurrentTheme(): string {
  const read = () => document.documentElement.dataset.theme ?? "dark";
  const [theme, setTheme] = useState(read);
  useEffect(() => {
    const observer = new MutationObserver(() => setTheme(read()));
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });
    return () => observer.disconnect();
  }, []);
  return theme;
}

export function FilePreviewHighlight({ code, path, mode = "normal" }: FilePreviewHighlightProps) {
  const { t } = useTranslation();
  const [html, setHtml] = useState("");
  const appTheme = useCurrentTheme();

  useEffect(() => {
    let alive = true;
    const shikiTheme = appTheme === "light" ? "one-light" : "one-dark-pro";
    codeToHtml(code, {
      lang: languageFromPath(path),
      theme: shikiTheme,
      transformers: [lineNumberTransformer(mode)],
    })
      .then((result) => {
        if (alive) setHtml(result);
      })
      .catch(() => {
        if (alive) setHtml(escapeFallback(code));
      });
    return () => { alive = false; };
  }, [code, path, mode, appTheme]);

  if (!html) return <div className="fp-empty">{t("filePreview.highlighting")}</div>;
  return <div className="fp-shiki" dangerouslySetInnerHTML={{ __html: html }} />;
}

function lineNumberTransformer(mode: FilePreviewHighlightProps["mode"]) {
  return {
    line(node: { properties?: Record<string, unknown> }, line: number) {
      node.properties = {
        ...node.properties,
        "data-line": String(line),
        class: mode === "normal" ? "line" : `line is-${mode}`,
      };
    },
  };
}

function escapeFallback(code: string): string {
  const escaped = code
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
  return `<pre><code>${escaped}</code></pre>`;
}

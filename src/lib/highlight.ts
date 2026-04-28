import { createLowlight, common } from "lowlight";
import { toHtml } from "hast-util-to-html";
import { languageFromPath } from "./code-language";

const lowlight = createLowlight(common);

const LANG_ALIASES: Record<string, string> = {
  jsx: "javascript",
  tsx: "typescript",
  toml: "ini",
  html: "xml",
};

export function highlightLines(code: string, path: string): string[] {
  const lang = languageFromPath(path);
  const resolved = LANG_ALIASES[lang] ?? lang;
  const registered = lowlight.registered(resolved);

  const lines = code.split("\n");
  if (!registered) return lines;

  const tree = lowlight.highlight(resolved, code);
  const html = toHtml(tree);
  return html.split("\n");
}

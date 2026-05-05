import { open } from "@tauri-apps/plugin-shell";
import { LinkPreviewCard } from "@/components/agent-local/link-preview-card";

const MAX_PREVIEWS = 5;

function isPreviewEnabled(): boolean {
  return localStorage.getItem("clgo-link-preview") !== "false";
}

interface LinkifyResult {
  text: React.ReactNode[];
  previews: React.ReactNode | null;
}

function stripUrlsAndClean(text: string): { cleaned: string; urls: string[] } {
  const regex = /https?:\/\/[^\s<>"')\]]+/g;
  const urls: string[] = [];
  const seen = new Set<string>();
  const stripped = text.replace(regex, (url) => {
    if (!seen.has(url) && urls.length < MAX_PREVIEWS) {
      seen.add(url);
      urls.push(url);
    }
    return "";
  });
  const cleaned = stripped
    .split("\n")
    .map((line) => line.replace(/^ +| +$/g, ""))
    .join("\n")
    .replace(/\n{3,}/g, "\n\n")
    .replace(/^\n+/, "")
    .replace(/\n+$/, "");
  return { cleaned, urls };
}

export function linkifyWithPreviews(text: string): LinkifyResult {
  const showPreview = isPreviewEnabled();

  if (!showPreview) {
    return { text: linkifyText(text), previews: null };
  }

  const { cleaned, urls } = stripUrlsAndClean(text);
  const textNodes = cleaned.length > 0 ? [cleaned] : [];
  const previewCards = urls.map((url) => (
    <LinkPreviewCard key={`preview-${url}`} url={url} />
  ));
  const previews = previewCards.length > 0 ? (
    <div className="chat-previews-block">{previewCards}</div>
  ) : null;

  return { text: textNodes, previews };
}

function linkifyText(text: string): React.ReactNode[] {
  const regex = /https?:\/\/[^\s<>"')\]]+/g;
  const parts: React.ReactNode[] = [];
  let lastIndex = 0;
  let match;

  while ((match = regex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      parts.push(text.slice(lastIndex, match.index));
    }
    const url = match[0];
    parts.push(
      <a
        key={match.index}
        className="chat-link"
        href={url}
        title={url}
        target="_blank"
        rel="noopener noreferrer"
        onClick={(e) => { e.preventDefault(); void open(url); }}
      >
        {url}
      </a>
    );
    lastIndex = match.index + url.length;
  }

  if (lastIndex < text.length) {
    parts.push(text.slice(lastIndex));
  }

  return parts.length > 0 ? parts : [text];
}

export function linkify(text: string): React.ReactNode[] {
  const { text: nodes, previews } = linkifyWithPreviews(text);
  if (previews) return [...nodes, previews];
  return nodes;
}

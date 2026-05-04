import apifyIconSvg from "@/assets/Apify-2/Apify-icon.svg?raw";
import apifyTextSvg from "@/assets/Apify-2/Apify-text.svg?raw";
import canvaIconSvg from "@/assets/Canva/canva-icon.svg?raw";
import canvaTextSvg from "@/assets/Canva/Canva-text.svg?raw";
import context7Png from "@/assets/context7/context7.png";
import figmaSvg from "@/assets/Figma/Figma.svg?raw";
import githubIconSvg from "@/assets/github/github.svg?raw";
import githubTextSvg from "@/assets/github/github-text.svg?raw";
import huggingfaceSvg from "@/assets/hugging-face/huggingface.svg?raw";
import imessageSvg from "@/assets/IMessage/IMessage_logo.svg?raw";
import linearIconSvg from "@/assets/Linear/Linear-icon.svg?raw";
import linearTextSvg from "@/assets/Linear/Linear-text.svg?raw";
import lucidPng from "@/assets/lucid/lucid.png";
import notionIconSvg from "@/assets/Notion/Notion-icon.svg?raw";
import notionTextSvg from "@/assets/Notion/Notion-text.svg?raw";
import producthuntSvg from "@/assets/Product_Hunt/Product-hunt.svg?raw";
import redditIconSvg from "@/assets/Reddit/Reddit-icon.svg?raw";
import redditTextSvg from "@/assets/Reddit/Reddit-text.svg?raw";
import sentryIconSvg from "@/assets/Sentry/Sentry-icon.svg?raw";
import sentryTextSvg from "@/assets/Sentry/Sentry-text.svg?raw";
import slackIconSvg from "@/assets/Slack-2/Slack-icon.svg?raw";
import slackTextSvg from "@/assets/Slack-2/Slack-text.svg?raw";
import vercelIconSvg from "@/assets/Vercel/Vercel-icon.svg?raw";
import vercelTextSvg from "@/assets/Vercel/Vercel-text.svg?raw";

function scopeSvg(raw: string, prefix: string): string {
  let s = raw;
  s = s.replace(/\bid="([^"]+)"/g, `id="${prefix}$1"`);
  s = s.replace(/url\(#([^)]+)\)/g, `url(#${prefix}$1)`);
  s = s.replace(/xlink:href="#([^"]+)"/g, `xlink:href="#${prefix}$1"`);
  s = s.replace(/href="#([^"]+)"/g, `href="#${prefix}$1"`);
  s = s.replace(/class="([^"]+)"/g, (_match, classes: string) => {
    const scoped = classes.split(/\s+/).map((c) => `${prefix}${c}`).join(" ");
    return `class="${scoped}"`;
  });
  s = s.replace(/\.st(\d+)\s*\{/g, `.${prefix}st$1{`);
  return s;
}

type McpIconVariant = "icon" | "text";

interface SvgEntry { kind: "svg"; icon: string; text: string; hasText?: boolean }
interface ImgEntry { kind: "img"; src: string }

type McpIconEntry = SvgEntry | ImgEntry;

const MCP_ICONS: Record<string, McpIconEntry> = {
  apify:       { kind: "svg", icon: scopeSvg(apifyIconSvg, "apify-i-"), text: scopeSvg(apifyTextSvg, "apify-t-"), hasText: true },
  canva:       { kind: "svg", icon: scopeSvg(canvaIconSvg, "canva-i-"), text: scopeSvg(canvaTextSvg, "canva-t-"), hasText: true },
  context7:    { kind: "img", src: context7Png },
  figma:       { kind: "svg", icon: scopeSvg(figmaSvg, "figma-"), text: scopeSvg(figmaSvg, "figma-t-") },
  github:      { kind: "svg", icon: scopeSvg(githubIconSvg, "gh-i-"), text: scopeSvg(githubTextSvg, "gh-t-"), hasText: true },
  huggingface: { kind: "svg", icon: scopeSvg(huggingfaceSvg, "hf-"), text: scopeSvg(huggingfaceSvg, "hf-t-") },
  imessage:    { kind: "svg", icon: scopeSvg(imessageSvg, "im-"), text: scopeSvg(imessageSvg, "im-t-") },
  linear:      { kind: "svg", icon: scopeSvg(linearIconSvg, "lin-i-"), text: scopeSvg(linearTextSvg, "lin-t-"), hasText: true },
  lucid:       { kind: "img", src: lucidPng },
  notion:      { kind: "svg", icon: scopeSvg(notionIconSvg, "not-i-"), text: scopeSvg(notionTextSvg, "not-t-"), hasText: true },
  producthunt: { kind: "svg", icon: scopeSvg(producthuntSvg, "ph-"), text: scopeSvg(producthuntSvg, "ph-t-") },
  reddit:      { kind: "svg", icon: scopeSvg(redditIconSvg, "red-i-"), text: scopeSvg(redditTextSvg, "red-t-"), hasText: true },
  sentry:      { kind: "svg", icon: scopeSvg(sentryIconSvg, "sen-i-"), text: scopeSvg(sentryTextSvg, "sen-t-"), hasText: true },
  slack:       { kind: "svg", icon: scopeSvg(slackIconSvg, "slk-i-"), text: scopeSvg(slackTextSvg, "slk-t-"), hasText: true },
  vercel:      { kind: "svg", icon: scopeSvg(vercelIconSvg, "ver-i-"), text: scopeSvg(vercelTextSvg, "ver-t-"), hasText: true },
};

export function mcpHasTextIcon(connectorId: string): boolean {
  const entry = MCP_ICONS[connectorId];
  return entry?.kind === "svg" && !!entry.hasText;
}

interface McpIconProps {
  connectorId: string;
  displayName: string;
  size?: number;
  variant?: McpIconVariant;
  textWidth?: boolean;
}

export function McpIcon({ connectorId, displayName, size = 40, variant = "icon", textWidth }: McpIconProps) {
  const entry = MCP_ICONS[connectorId];

  if (!entry) {
    return <McpIconFallback displayName={displayName} connectorId={connectorId} size={size} />;
  }

  if (entry.kind === "img") {
    return (
      <img
        src={entry.src}
        alt={displayName}
        style={{ width: size, height: size, borderRadius: 8, objectFit: "contain", flexShrink: 0 }}
      />
    );
  }

  const raw = variant === "text" ? entry.text : entry.icon;

  if (textWidth) {
    return (
      <span
        className="mcp-icon-inline mcp-icon-text"
        style={{ height: size, display: "inline-flex", flexShrink: 0 }}
        dangerouslySetInnerHTML={{ __html: raw }}
      />
    );
  }

  return (
    <span
      className="mcp-icon-inline"
      style={{ width: size, height: size, display: "inline-flex", flexShrink: 0 }}
      dangerouslySetInnerHTML={{ __html: raw }}
    />
  );
}

function McpIconFallback({ displayName, connectorId, size }: { displayName: string; connectorId: string; size: number }) {
  const color = colorFor(connectorId);
  return (
    <div style={{
      width: size, height: size, borderRadius: 8,
      background: `${color}22`, color,
      display: "flex", alignItems: "center", justifyContent: "center",
      fontWeight: 700, fontSize: size * 0.45, flexShrink: 0,
    }}>
      {displayName.charAt(0).toUpperCase()}
    </div>
  );
}

function colorFor(id: string): string {
  const palette = [
    "#f97316", "#3b82f6", "#10b981", "#8b5cf6", "#ec4899",
    "#eab308", "#06b6d4", "#ef4444", "#84cc16", "#a855f7",
  ];
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = (hash * 31 + id.charCodeAt(i)) >>> 0;
  }
  return palette[hash % palette.length];
}

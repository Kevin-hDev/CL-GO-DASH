import groqIcon from "@/assets/groq.svg";
import geminiIcon from "@/assets/GEMINI.png";
import mistralIcon from "@/assets/mistral-color.svg";
import cerebrasIcon from "@/assets/cerebras-color.svg";
import openrouterIcon from "@/assets/openrouter.svg";
import openaiIcon from "@/assets/openai.svg";
import deepseekIcon from "@/assets/deepseek-color.svg";
import braveIcon from "@/assets/BRAVE.jpeg";
import exaIcon from "@/assets/exa-color.svg";
import firecrawlIcon from "@/assets/FIRECRAWL.png";
import serpapiIcon from "@/assets/SERPAPI.png";
import googleSearchIcon from "@/assets/google-search.svg";

interface IconEntry {
  src: string;
  mono?: boolean;
}

const ICONS: Record<string, IconEntry> = {
  groq: { src: groqIcon, mono: true },
  google: { src: geminiIcon },
  mistral: { src: mistralIcon },
  cerebras: { src: cerebrasIcon },
  openrouter: { src: openrouterIcon, mono: true },
  openai: { src: openaiIcon, mono: true },
  deepseek: { src: deepseekIcon },
  brave: { src: braveIcon },
  exa: { src: exaIcon },
  firecrawl: { src: firecrawlIcon },
  serpapi: { src: serpapiIcon },
  google_cse: { src: googleSearchIcon },
};

interface ProviderIconProps {
  providerId: string;
  displayName: string;
  size?: number;
}

export function ProviderIcon({ providerId, displayName, size = 40 }: ProviderIconProps) {
  const entry = ICONS[providerId];

  if (!entry) {
    const color = colorFor(providerId);
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

  return (
    <img
      src={entry.src}
      alt={displayName}
      className={entry.mono ? "provider-icon-mono" : undefined}
      style={{ width: size, height: size, borderRadius: 8, objectFit: "contain", flexShrink: 0 }}
    />
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

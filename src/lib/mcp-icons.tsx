import redditIcon from "@/assets/REDDIT.png";

interface McpIconEntry {
  src: string;
  mono?: boolean;
}

const MCP_ICONS: Record<string, McpIconEntry> = {
  reddit: { src: redditIcon },
};

interface McpIconProps {
  connectorId: string;
  displayName: string;
  size?: number;
}

export function McpIcon({ connectorId, displayName, size = 40 }: McpIconProps) {
  const entry = MCP_ICONS[connectorId];

  if (!entry) {
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

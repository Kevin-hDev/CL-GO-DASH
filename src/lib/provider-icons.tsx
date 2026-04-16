// Placeholder pour les icônes providers — Kevin fournira les vrais SVG.
// En attendant : cercle coloré avec la première lettre du nom.

interface ProviderIconProps {
  providerId: string;
  displayName: string;
  size?: number;
}

/**
 * Palette cohérente par provider (fallback tant que les SVG ne sont pas fournis).
 * Les couleurs sont dérivées du hash du provider_id pour être stables.
 */
function colorFor(id: string): string {
  const palette = [
    "#f97316", // orange
    "#3b82f6", // blue
    "#10b981", // emerald
    "#8b5cf6", // violet
    "#ec4899", // pink
    "#eab308", // yellow
    "#06b6d4", // cyan
    "#ef4444", // red
    "#84cc16", // lime
    "#a855f7", // purple
  ];
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = (hash * 31 + id.charCodeAt(i)) >>> 0;
  }
  return palette[hash % palette.length];
}

export function ProviderIcon({
  providerId,
  displayName,
  size = 40,
}: ProviderIconProps) {
  const color = colorFor(providerId);
  const initial = displayName.charAt(0).toUpperCase();

  return (
    <div
      style={{
        width: size,
        height: size,
        borderRadius: 8,
        background: `${color}22`,
        color,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        fontWeight: 700,
        fontSize: size * 0.45,
        flexShrink: 0,
      }}
    >
      {initial}
    </div>
  );
}

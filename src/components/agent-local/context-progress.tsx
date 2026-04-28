import "./context-progress.css";

interface ContextProgressProps {
  used: number;
  max: number;
}

type ColorKey = "blue" | "yellow" | "orange" | "red";

function colorForPercentage(p: number): ColorKey {
  if (p >= 90) return "red";
  if (p >= 70) return "orange";
  if (p >= 55) return "yellow";
  return "blue";
}

const FILL_COLORS: Record<ColorKey, string> = {
  blue: "var(--signal-info)",
  yellow: "var(--signal-warning)",
  orange: "var(--tool-bash)",
  red: "var(--signal-error)",
};

function formatTokens(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
  return String(n);
}

const SIZE = 18;
const STROKE = 3;
const RADIUS = (SIZE - STROKE) / 2;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

export function ContextProgress({ used, max }: ContextProgressProps) {
  if (!max || max <= 0) return null;
  const percentage = Math.min((used / max) * 100, 100);
  const colorKey = colorForPercentage(percentage);
  const offset = CIRCUMFERENCE - (percentage / 100) * CIRCUMFERENCE;
  const pctDisplay = percentage < 1 ? "0" : percentage.toFixed(0);
  const tooltip = `${formatTokens(used)} / ${formatTokens(max)} tokens (${pctDisplay}%)`;

  return (
    <div className="context-ring" data-tooltip={tooltip}>
      <svg width={SIZE} height={SIZE} viewBox={`0 0 ${SIZE} ${SIZE}`}>
        <circle
          className="context-ring-track"
          cx={SIZE / 2}
          cy={SIZE / 2}
          r={RADIUS}
          strokeWidth={STROKE}
        />
        <circle
          className="context-ring-fill"
          cx={SIZE / 2}
          cy={SIZE / 2}
          r={RADIUS}
          strokeWidth={STROKE}
          stroke={FILL_COLORS[colorKey]}
          strokeDasharray={CIRCUMFERENCE}
          strokeDashoffset={offset}
        />
      </svg>
    </div>
  );
}

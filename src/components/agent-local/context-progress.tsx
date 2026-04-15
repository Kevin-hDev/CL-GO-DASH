import "./context-progress.css";

interface ContextProgressProps {
  used: number;
  max: number;
}

type ColorKey = "green" | "yellow" | "orange" | "red";

function colorForPercentage(p: number): ColorKey {
  if (p >= 90) return "red";
  if (p >= 70) return "orange";
  if (p >= 55) return "yellow";
  return "green";
}

function formatTokens(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
  return String(n);
}

export function ContextProgress({ used, max }: ContextProgressProps) {
  if (!max || max <= 0) return null;
  const percentage = Math.min((used / max) * 100, 100);
  const colorKey = colorForPercentage(percentage);
  const tooltip = `${formatTokens(used)} / ${formatTokens(max)} tokens`;

  return (
    <div className="context-progress" title={tooltip} data-color={colorKey}>
      <div
        className="context-progress-fill"
        style={{ width: `${percentage}%` }}
      />
      <span className="context-progress-text">
        {percentage < 1 ? "0" : percentage.toFixed(0)}%
      </span>
    </div>
  );
}

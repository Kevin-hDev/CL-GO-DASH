import "./conversation.css";

interface ContextBarProps {
  percentage: number;
  color: string;
}

export function ContextBar({ percentage, color }: ContextBarProps) {
  const width = Math.min(percentage, 100);

  return (
    <div className="context-bar">
      <div
        className="context-bar-fill"
        style={{ width: `${width}%`, backgroundColor: color }}
      />
    </div>
  );
}

import "./round-toggle.css";

interface RoundToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  title?: string;
}

export function RoundToggle({ checked, onChange, title }: RoundToggleProps) {
  return (
    <label className="round-toggle" title={title}>
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
      />
      <div className="rt-layer rt-base">ON</div>
      <div className="rt-layer rt-split">
        <div />
        <div />
      </div>
      <div className="rt-layer rt-slide-r" />
      <div className="rt-layer rt-slide-l">OFF</div>
    </label>
  );
}

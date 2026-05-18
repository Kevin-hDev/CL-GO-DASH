import "./round-toggle.css";

interface RoundToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  title?: string;
  disabled?: boolean;
}

export function RoundToggle({ checked, onChange, title, disabled = false }: RoundToggleProps) {
  return (
    <label className={`round-toggle${disabled ? " is-disabled" : ""}`} title={title}>
      <input
        type="checkbox"
        checked={checked}
        disabled={disabled}
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

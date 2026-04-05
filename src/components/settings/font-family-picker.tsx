import { FONT_FAMILIES, type FontFamilyId } from "@/hooks/use-settings";

interface FontFamilyPickerProps {
  value: FontFamilyId;
  onChange: (id: FontFamilyId) => void;
}

export function FontFamilyPicker({ value, onChange }: FontFamilyPickerProps) {
  return (
    <div>
      <label style={{
        display: "block", fontSize: "var(--text-xs)", color: "var(--ink-muted)",
        textTransform: "uppercase", letterSpacing: "0.5px", marginBottom: 12,
      }}>
        Font family
      </label>
      <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
        {FONT_FAMILIES.map((font) => (
          <div
            key={font.id}
            onClick={() => onChange(font.id)}
            style={{
              display: "flex", alignItems: "center", justifyContent: "space-between",
              padding: "12px 16px",
              borderRadius: "var(--radius-sm)", cursor: "pointer",
              border: font.id === value ? "1px solid var(--pulse)" : "1px solid var(--edge)",
              background: font.id === value ? "var(--pulse-muted)" : "transparent",
              transition: "all 200ms ease-out",
            }}
          >
            <span style={{
              fontFamily: font.value,
              fontSize: "var(--text-sm)",
              color: font.id === value ? "var(--pulse)" : "var(--ink)",
            }}>
              {font.label}
            </span>
            <span style={{
              fontFamily: font.value,
              fontSize: "var(--text-xs)",
              color: "var(--ink-faint)",
            }}>
              The quick brown fox
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}

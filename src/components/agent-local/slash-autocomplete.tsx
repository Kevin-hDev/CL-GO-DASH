import type { SkillInfo } from "@/types/agent";

interface SlashAutocompleteProps {
  skills: SkillInfo[];
  onSelect: (skill: SkillInfo) => void;
}

export function SlashAutocomplete({ skills, onSelect }: SlashAutocompleteProps) {
  if (skills.length < 1) return null;

  return (
    <div style={{
      position: "absolute", bottom: "100%", marginBottom: 4, left: 0,
      width: 280, maxHeight: 200, overflowY: "auto",
      borderRadius: "var(--radius-md)", border: "1px solid var(--edge)",
      background: "var(--shell)", boxShadow: "var(--shadow-card)", zIndex: 50,
    }}>
      {skills.map((skill) => (
        <div
          key={skill.name}
          onClick={() => onSelect(skill)}
          style={{
            padding: "var(--space-sm) var(--space-md)", cursor: "pointer",
            transition: "background var(--ease-smooth)",
          }}
          onMouseEnter={(e) => { e.currentTarget.style.background = "var(--pulse-muted)"; }}
          onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}
        >
          <div style={{ fontSize: "var(--text-sm)", color: "var(--ink)" }}>
            /{skill.name}
          </div>
          {skill.description && (
            <div style={{
              fontSize: "var(--text-xs)", color: "var(--ink-faint)",
              overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
            }}>
              {skill.description}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

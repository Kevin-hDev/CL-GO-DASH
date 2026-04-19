import type { SkillInfo } from "@/types/agent";

interface SkillBadgeProps {
  skill: SkillInfo;
  onRemove: () => void;
}

export function SkillBadge({ skill, onRemove }: SkillBadgeProps) {
  return (
    <span style={{
      display: "inline-flex",
      alignItems: "center",
      gap: 4,
      padding: "2px 8px 2px 6px",
      borderRadius: "var(--radius-sm)",
      background: "var(--pulse-muted)",
      color: "var(--pulse)",
      fontSize: "var(--text-sm)",
      fontWeight: 500,
      lineHeight: 1.4,
    }}>
      <svg width="14" height="14" viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeWidth="1.5">
        <rect x="3" y="3" width="14" height="14" rx="3" />
        <path d="M8 7l4 3-4 3" />
      </svg>
      {skill.name}
      <button
        onClick={(e) => { e.stopPropagation(); onRemove(); }}
        style={{
          background: "none",
          border: "none",
          cursor: "pointer",
          color: "var(--pulse)",
          padding: 0,
          marginLeft: 2,
          fontSize: "var(--text-sm)",
          lineHeight: 1,
          opacity: 0.6,
        }}
        onMouseEnter={(e) => { e.currentTarget.style.opacity = "1"; }}
        onMouseLeave={(e) => { e.currentTarget.style.opacity = "0.6"; }}
      >
        ×
      </button>
    </span>
  );
}

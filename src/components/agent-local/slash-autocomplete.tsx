import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import type { SkillInfo } from "@/types/agent";
import "./slash-autocomplete.css";

interface SlashAutocompleteProps {
  skills: SkillInfo[];
  activeIndex: number;
  onSelect: (skill: SkillInfo) => void;
}

function SkillIcon() {
  return (
    <svg className="slash-item-icon" viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeWidth="1.5">
      <rect x="3" y="3" width="14" height="14" rx="3" />
      <path d="M8 7l4 3-4 3" />
    </svg>
  );
}

export function SlashAutocomplete({ skills, activeIndex, onSelect }: SlashAutocompleteProps) {
  const { t } = useTranslation();
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const active = listRef.current?.children[activeIndex] as HTMLElement | undefined;
    active?.scrollIntoView({ block: "nearest" });
  }, [activeIndex]);

  if (skills.length < 1) {
    return (
      <div className="slash-dropdown">
        <div className="slash-empty">{t("skills.noResults")}</div>
      </div>
    );
  }

  return (
    <div className="slash-dropdown" ref={listRef}>
      {skills.map((skill, i) => (
        <div
          key={skill.path}
          className={`slash-item ${i === activeIndex ? "active" : ""}`}
          onClick={() => onSelect(skill)}
        >
          <SkillIcon />
          <div className="slash-item-body">
            <span className="slash-item-name">{skill.name}</span>
            {skill.description && (
              <span className="slash-item-desc">{skill.description}</span>
            )}
          </div>
          <span className="slash-item-source">
            {t(`skills.source${skill.source === "project" ? "Project" : "User"}`)}
          </span>
        </div>
      ))}
    </div>
  );
}

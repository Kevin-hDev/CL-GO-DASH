import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SkillInfo } from "@/types/agent";

export function useSlashCommands() {
  const [skills, setSkills] = useState<SkillInfo[]>([]);
  const [showDropdown, setShowDropdown] = useState(false);
  const [filter, setFilter] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);

  useEffect(() => {
    invoke<SkillInfo[]>("list_skills")
      .then(setSkills)
      .catch((e: unknown) => console.warn("Erreur chargement skills:", e));
  }, []);

  const filtered = filter
    ? skills.filter((s) =>
        s.name.toLowerCase().includes(filter.toLowerCase())
        || s.description.toLowerCase().includes(filter.toLowerCase()))
    : skills;

  const handleInput = useCallback((text: string) => {
    const lastSlash = text.lastIndexOf("/");
    if (lastSlash >= 0) {
      const before = lastSlash === 0 ? "" : text[lastSlash - 1];
      const isWordStart = before === "" || before === " " || before === "\n";
      const afterSlash = text.slice(lastSlash + 1);
      const isSlashCommand = isWordStart && !afterSlash.includes(" ") && !afterSlash.includes("/");
      if (isSlashCommand) {
        setShowDropdown(true);
        setFilter(afterSlash);
        setActiveIndex(0);
        return;
      }
    }
    setShowDropdown(false);
    setFilter("");
  }, []);

  const selectSkill = useCallback(async (skill: SkillInfo) => {
    setShowDropdown(false);
    setFilter("");
    setActiveIndex(0);
    try {
      const content = await invoke<string>("load_skill", { name: skill.name });
      return { content, skill };
    } catch (e: unknown) {
      console.warn("Erreur chargement skill:", e);
      return null;
    }
  }, []);

  const moveUp = useCallback(() => {
    setActiveIndex((i) => (i > 0 ? i - 1 : filtered.length - 1));
  }, [filtered.length]);

  const moveDown = useCallback(() => {
    setActiveIndex((i) => (i < filtered.length - 1 ? i + 1 : 0));
  }, [filtered.length]);

  const close = useCallback(() => {
    setShowDropdown(false);
    setFilter("");
    setActiveIndex(0);
  }, []);

  return {
    skills: filtered,
    showDropdown,
    activeIndex,
    handleInput,
    selectSkill,
    moveUp,
    moveDown,
    close,
  };
}

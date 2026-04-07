import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SkillInfo } from "@/types/agent";

export function useSlashCommands() {
  const [skills, setSkills] = useState<SkillInfo[]>([]);
  const [showDropdown, setShowDropdown] = useState(false);
  const [filter, setFilter] = useState("");

  useEffect(() => {
    invoke<SkillInfo[]>("list_skills")
      .then(setSkills)
      .catch((e: unknown) => console.warn("Erreur chargement skills:", e));
  }, []);

  const filtered = filter
    ? skills.filter((s) => s.name.toLowerCase().includes(filter.toLowerCase()))
    : skills;

  const handleInput = useCallback((text: string) => {
    if (text.startsWith("/")) {
      setShowDropdown(true);
      setFilter(text.slice(1));
    } else {
      setShowDropdown(false);
      setFilter("");
    }
  }, []);

  const selectSkill = useCallback(async (skill: SkillInfo) => {
    setShowDropdown(false);
    setFilter("");
    try {
      const content = await invoke<string>("load_skill", { name: skill.name });
      return content;
    } catch (e: unknown) {
      console.warn("Erreur chargement skill:", e);
      return null;
    }
  }, []);

  const close = useCallback(() => {
    setShowDropdown(false);
    setFilter("");
  }, []);

  return { skills: filtered, showDropdown, handleInput, selectSkill, close };
}

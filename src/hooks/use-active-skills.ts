import { useState, useCallback, useRef } from "react";
import type { SkillInfo } from "@/types/agent";
import type { useSlashCommands } from "@/hooks/use-slash-commands";

export interface ActiveSkillsState {
  activeSkills: SkillInfo[];
  skillContentsRef: React.RefObject<Map<string, string>>;
  handleSelectSkill: (skill: SkillInfo) => Promise<void>;
  handleRemoveSkill: (name: string) => void;
  getSkillsPayload: () => { name: string; content: string }[] | undefined;
  clearSkills: () => void;
}

export function useActiveSkills(
  slash: ReturnType<typeof useSlashCommands>,
  text: string,
  setText: (v: string) => void,
): ActiveSkillsState {
  const [activeSkills, setActiveSkills] = useState<SkillInfo[]>([]);
  const skillContentsRef = useRef<Map<string, string>>(new Map());

  const handleSelectSkill = useCallback(async (skill: SkillInfo) => {
    if (activeSkills.some((s) => s.name === skill.name)) return;
    const result = await slash.selectSkill(skill);
    if (result) {
      setActiveSkills((prev) => [...prev, result.skill]);
      skillContentsRef.current.set(result.skill.name, result.content);
      const lastSlash = text.lastIndexOf("/");
      setText(lastSlash > 0 ? text.slice(0, lastSlash).trimEnd() : "");
    }
  }, [slash, text, setText, activeSkills]);

  const handleRemoveSkill = useCallback((name: string) => {
    setActiveSkills((prev) => prev.filter((s) => s.name !== name));
    skillContentsRef.current.delete(name);
  }, []);

  const getSkillsPayload = useCallback(() => {
    if (activeSkills.length === 0) return undefined;
    return activeSkills.map((s) => ({
      name: s.name,
      content: skillContentsRef.current.get(s.name) ?? "",
    }));
  }, [activeSkills]);

  const clearSkills = useCallback(() => {
    setActiveSkills([]);
    skillContentsRef.current.clear();
  }, []);

  return {
    activeSkills, skillContentsRef,
    handleSelectSkill, handleRemoveSkill,
    getSkillsPayload, clearSkills,
  };
}

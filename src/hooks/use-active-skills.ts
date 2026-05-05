import { useState, useCallback, useRef } from "react";
import type { SkillInfo } from "@/types/agent";
import type { useSlashCommands } from "@/hooks/use-slash-commands";
import type { SlashItem } from "@/hooks/use-slash-commands";

export interface ActiveSkillsState {
  activeSkills: SkillInfo[];
  skillContentsRef: React.RefObject<Map<string, string>>;
  handleSelectSkill: (item: SlashItem) => Promise<void>;
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

  const handleSelectSkill = useCallback(async (item: SlashItem) => {
    const result = await slash.selectItem(item);
    if (!result) return;

    if ("builtIn" in result) {
      setText("/" + result.builtIn.name);
      return;
    }

    const { skill, content } = result;
    if (activeSkills.some((s) => s.name === skill.name)) return;
    setActiveSkills((prev) => [...prev, skill]);
    skillContentsRef.current.set(skill.name, content);
    const lastSlash = text.lastIndexOf("/");
    setText(lastSlash > 0 ? text.slice(0, lastSlash).trimEnd() : "");
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

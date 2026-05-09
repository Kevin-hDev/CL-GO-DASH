import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SkillInfo } from "@/types/agent";
import { useFsEvent } from "@/hooks/use-fs-event";

export interface BuiltInCommand {
  name: string;
  description: string;
  source: "built-in";
  path: string;
}

const BUILT_IN_COMMANDS: BuiltInCommand[] = [
  {
    name: "compress",
    description: "Compress conversation context manually",
    source: "built-in",
    path: "__built-in__/compress",
  },
];

export type SlashItem = SkillInfo | BuiltInCommand;

export function isBuiltIn(item: SlashItem): item is BuiltInCommand {
  return item.source === "built-in";
}

export function useSlashCommands() {
  const [skills, setSkills] = useState<SkillInfo[]>([]);
  const [showDropdown, setShowDropdown] = useState(false);
  const [filter, setFilter] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);

  const loadSkills = useCallback(() => {
    invoke<SkillInfo[]>("list_skills")
      .then(setSkills)
      .catch((e: unknown) => console.warn("Erreur chargement skills:", e));
  }, []);

  useEffect(() => { loadSkills(); }, [loadSkills]);

  useFsEvent("fs:skills-changed", loadSkills);

  const allItems: SlashItem[] = filter
    ? [
        ...BUILT_IN_COMMANDS.filter(
          (c) =>
            c.name.toLowerCase().includes(filter.toLowerCase()) ||
            c.description.toLowerCase().includes(filter.toLowerCase()),
        ),
        ...skills.filter(
          (s) =>
            s.name.toLowerCase().includes(filter.toLowerCase()) ||
            s.description.toLowerCase().includes(filter.toLowerCase()),
        ),
      ]
    : [...BUILT_IN_COMMANDS, ...skills];

  const handleInput = useCallback((text: string) => {
    const lastSlash = text.lastIndexOf("/");
    if (lastSlash >= 0) {
      const before = lastSlash === 0 ? "" : text[lastSlash - 1];
      const isWordStart = before === "" || before === " " || before === "\n";
      const afterSlash = text.slice(lastSlash + 1);
      const isSlashCommand =
        isWordStart && !afterSlash.includes(" ") && !afterSlash.includes("/");
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

  const selectItem = useCallback(
    async (item: SlashItem): Promise<{ content: string; skill: SkillInfo } | { builtIn: BuiltInCommand } | null> => {
      setShowDropdown(false);
      setFilter("");
      setActiveIndex(0);
      if (isBuiltIn(item)) {
        return { builtIn: item };
      }
      try {
        const content = await invoke<string>("load_skill", { name: item.name });
        return { content, skill: item };
      } catch (e: unknown) {
        console.warn("Erreur chargement skill:", e);
        return null;
      }
    },
    [],
  );

  const moveUp = useCallback(() => {
    setActiveIndex((i) => (i > 0 ? i - 1 : allItems.length - 1));
  }, [allItems.length]);

  const moveDown = useCallback(() => {
    setActiveIndex((i) => (i < allItems.length - 1 ? i + 1 : 0));
  }, [allItems.length]);

  const close = useCallback(() => {
    setShowDropdown(false);
    setFilter("");
    setActiveIndex(0);
  }, []);

  return {
    skills: allItems,
    showDropdown,
    activeIndex,
    handleInput,
    selectSkill,
    selectItem,
    moveUp,
    moveDown,
    close,
  };
}

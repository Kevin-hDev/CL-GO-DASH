import { describe, expect, it } from "vitest";
import type { SkillInfo } from "@/types/agent";
import { activeSkillsInText, replaceSlashToken } from "@/lib/skill-text";

const skills: SkillInfo[] = [
  { name: "context7-docs", description: "", path: "", source: "user" },
  { name: "frontend-design", description: "", path: "", source: "user" },
];

describe("skill-text", () => {
  it("remplace uniquement le token slash en cours", () => {
    const text = "Utilise /cont pour vérifier";
    expect(replaceSlashToken(text, "context7-docs")).toBe(
      "Utilise /context7-docs pour vérifier",
    );
  });

  it("garde les skills actifs uniquement s'ils restent dans le texte", () => {
    const text = "Charge /context7-docs puis /frontend-design";
    expect(activeSkillsInText(text, skills).map((s) => s.name)).toEqual([
      "context7-docs",
      "frontend-design",
    ]);
  });

  it("ignore un skill supprimé du texte", () => {
    expect(activeSkillsInText("Pas de skill ici", skills)).toEqual([]);
  });
});

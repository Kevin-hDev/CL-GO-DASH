import { describe, expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import type { SkillInfo } from "@/types/agent";
import { activeSkillsInText, highlightSkillNodes, replaceSlashToken } from "@/lib/skill-text";

const skills: SkillInfo[] = [
  { name: "context7-docs", description: "", path: "", source: "user" },
  { name: "frontend-design", description: "", path: "", source: "user" },
  { name: "sharp-edges:sharp-edges", description: "", path: "", source: "user" },
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

  it("supporte les noms avec ponctuation sans regex dynamique", () => {
    const text = "Audit /sharp-edges:sharp-edges.";
    expect(activeSkillsInText(text, skills).map((s) => s.name)).toEqual([
      "sharp-edges:sharp-edges",
    ]);
  });

  it("ne détecte pas les tokens collés à un mot", () => {
    expect(activeSkillsInText("foo/context7-docs", skills)).toEqual([]);
  });

  it("highlightSkillNodes produit un chip inline sans slash visible", () => {
    const nodes = highlightSkillNodes(["Charge /context7-docs maintenant"], ["context7-docs"]);
    const html = renderToStaticMarkup(<>{nodes}</>);
    expect(html).toContain('class="skill-chip"');
    expect(html).toContain("<svg");
    expect(html).toContain(">context7-docs</span>");
    // Le slash ne doit pas apparaître dans le nom affiché
    expect(html).not.toContain(">context7-docs</span>/");
  });

  it("highlightSkillNodes rend les built-ins avec la classe built-in", () => {
    const nodes = highlightSkillNodes(
      ["Compresse avec /compress stp"],
      [],
      { builtInNames: ["compress"] },
    );
    const html = renderToStaticMarkup(<>{nodes}</>);
    expect(html).toContain('class="skill-chip skill-chip-built-in"');
    expect(html).toContain("<svg");
    expect(html).toContain(">compress</span>");
  });

  it("highlightSkillNodes n'ajoute pas de chip quand aucun token matché", () => {
    const nodes = highlightSkillNodes(["Texte sans skill"], ["context7-docs"]);
    const html = renderToStaticMarkup(<>{nodes}</>);
    expect(html).not.toContain('class="skill-chip"');
  });
});

import { describe, expect, it } from "vitest";
import { parseReleaseNotes } from "../update-release-notes";

describe("parseReleaseNotes", () => {
  it("lit les sections et bullets courtes", () => {
    const sections = parseReleaseNotes(`
### Features
- **Context usage** details
- \`Font size\` in pixels

### Fixes
- Settings apply at startup
`);

    expect(sections).toEqual([
      {
        title: "Features",
        items: ["Context usage details", "Font size in pixels"],
      },
      {
        title: "Fixes",
        items: ["Settings apply at startup"],
      },
    ]);
  });

  it("limite les notes sans tronquer les phrases", () => {
    const notes = Array.from({ length: 10 }, (_, i) => `- Item ${i}`).join("\n");
    const sections = parseReleaseNotes(notes);

    expect(sections[0]?.items).toHaveLength(6);
    expect(sections[0]?.items.join("\n")).not.toContain("…");
  });

  it("ignore les lignes trop longues", () => {
    const sections = parseReleaseNotes(`- ${"x".repeat(181)}`);

    expect(sections).toEqual([]);
  });
});

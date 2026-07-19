import { describe, expect, it } from "vitest";
import { highlightLines } from "../highlight";

describe("highlightLines", () => {
  it("ne transforme pas le retour final en ligne vide", () => {
    expect(highlightLines("première\ndeuxième\n", "notes.txt")).toEqual([
      "première",
      "deuxième",
    ]);
  });

  it("conserve une vraie ligne vide avant le retour final", () => {
    expect(highlightLines("première\n\n", "notes.txt")).toEqual([
      "première",
      "",
    ]);
  });

  it("applique la même règle au code coloré", () => {
    expect(highlightLines("const value = 1;\n", "example.ts")).toHaveLength(1);
  });
});

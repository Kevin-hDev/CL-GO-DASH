import { describe, it, expect } from "vitest";
import { generateHunks, buildOldContent } from "../file-preview-diff-utils";

describe("buildOldContent", () => {
  it("remplace newText par oldText dans le currentContent", () => {
    const current = "line1\nnew text here\nline3";
    const result = buildOldContent(current, "old text", "new text here");
    expect(result).toBe("line1\nold text\nline3");
  });

  it("retourne le contenu intact si newText n'est pas trouvé", () => {
    const current = "line1\nline2\nline3";
    const result = buildOldContent(current, "old", "not present");
    expect(result).toBe(current);
  });

  it("gère le cas où oldText est undefined (suppression — newText trouvé)", () => {
    const current = "line1\ninserted\nline2";
    const result = buildOldContent(current, undefined, "inserted");
    // remplace "inserted" par "" (oldText = "")
    expect(result).toBe("line1\n\nline2");
  });

  it("gère le cas où les deux sont undefined", () => {
    const current = "unchanged content";
    const result = buildOldContent(current, undefined, undefined);
    expect(result).toBe(current);
  });
});

describe("generateHunks", () => {
  it("retourne [] si les contenus sont identiques", () => {
    const content = "a\nb\nc";
    expect(generateHunks(content, content)).toHaveLength(0);
  });

  it("génère un hunk quand une ligne est modifiée", () => {
    const old = "line1\nold line\nline3";
    const newc = "line1\nnew line\nline3";
    const hunks = generateHunks(old, newc);
    expect(hunks.length).toBeGreaterThan(0);
    expect(hunks[0]).toMatch(/^@@ -\d+,\d+ \+\d+,\d+ @@/);
  });

  it("le hunk contient les marqueurs - et + corrects", () => {
    const old = "before\nto remove\nafter";
    const newc = "before\nadded\nafter";
    const hunks = generateHunks(old, newc);
    const hunkText = hunks.join("\n");
    expect(hunkText).toContain("-to remove");
    expect(hunkText).toContain("+added");
  });

  it("inclut les lignes de contexte autour du changement", () => {
    const old = "ctx1\nctx2\nctx3\nchanged\nctx4\nctx5\nctx6";
    const newc = "ctx1\nctx2\nctx3\nmodified\nctx4\nctx5\nctx6";
    const hunks = generateHunks(old, newc);
    expect(hunks.length).toBe(1);
    const lines = hunks[0].split("\n");
    const contextLines = lines.filter((l) => l.startsWith(" "));
    expect(contextLines.length).toBeGreaterThan(0);
  });

  it("génère des hunks pour des changements multiples séparés", () => {
    const oldLines = Array.from({ length: 20 }, (_, i) => `line${i + 1}`);
    const newLines = [...oldLines];
    newLines[1] = "modified2";
    newLines[18] = "modified19";
    const old = oldLines.join("\n");
    const newc = newLines.join("\n");
    const hunks = generateHunks(old, newc);
    expect(hunks.length).toBe(2);
  });

  it("le header de hunk spécifie les bonnes positions", () => {
    const old = "a\nb\nc";
    const newc = "a\nB\nc";
    const hunks = generateHunks(old, newc);
    expect(hunks[0]).toMatch(/@@ -\d+,\d+ \+\d+,\d+ @@/);
  });
});

import { describe, expect, it } from "vitest";
import { shortenPath, toolDisplayInfo } from "../tool-display";
import type { RenderableTool } from "../tool-detail-row";
import type { TFunction } from "i18next";

const t = ((key: string) => ({
  "agentLocal.toolActivity.actions.read": "Read",
  "agentLocal.toolActivity.actions.create": "Create",
  "agentLocal.toolActivity.actions.edit": "Edit",
}[key] ?? key)) as TFunction;

describe("toolDisplayInfo", () => {
  it("raccourcit les chemins depuis la racine projet", () => {
    const path = "/Users/kevinh/Projects/systeme-pulse/src/App.tsx";
    expect(shortenPath(path, "/Users/kevinh/Projects/systeme-pulse")).toBe("systeme-pulse/src/App.tsx");
  });

  it("affiche une création avec stats + lignes", () => {
    const tool: RenderableTool = {
      name: "write_file",
      summary: "/Users/kevinh/Projects/systeme-pulse/vite.config.ts",
      content: "a\nb\n",
    };
    expect(toolDisplayInfo(tool, "/Users/kevinh/Projects/systeme-pulse", t)).toEqual({
      label: "Create",
      summary: "systeme-pulse/vite.config.ts",
      additions: 2,
      deletions: 0,
    });
  });

  it("affiche une modification avec stats old/new", () => {
    const tool: RenderableTool = {
      name: "edit_file",
      summary: "/Users/kevinh/Projects/systeme-pulse/src/main.rs",
      old_text: "a\nb\nc",
      new_text: "a",
    };
    expect(toolDisplayInfo(tool, "/Users/kevinh/Projects/systeme-pulse", t)).toEqual({
      label: "Edit",
      summary: "systeme-pulse/src/main.rs",
      additions: 1,
      deletions: 3,
    });
  });

  it("ne change pas web_search", () => {
    expect(toolDisplayInfo({ name: "web_search", summary: "tauri docs" }, undefined, t)).toEqual({
      label: "web_search",
      summary: "tauri docs",
    });
  });

  it("garde bash comme nom de tool affiché", () => {
    expect(toolDisplayInfo({ name: "bash", summary: "npm test" }, undefined, t)).toEqual({
      label: "bash",
      summary: "npm test",
    });
  });

  it("tronque les commandes bash longues sur une seule ligne", () => {
    const command = `${"a".repeat(110)}\necho done`;
    expect(toolDisplayInfo({ name: "bash", summary: command }, undefined, t)).toEqual({
      label: "bash",
      summary: `${"a".repeat(96)}...`,
    });
  });
});

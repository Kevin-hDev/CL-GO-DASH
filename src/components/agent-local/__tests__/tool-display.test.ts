import { describe, expect, it } from "vitest";
import { shortenPath, toolDisplayInfo } from "../tool-display";
import type { RenderableTool } from "../tool-detail-row";
import type { TFunction } from "i18next";

const t = ((key: string) => ({
  "agentLocal.toolActivity.actions.read": "Read",
  "agentLocal.toolActivity.actions.create": "Create",
  "agentLocal.toolActivity.actions.edit": "Edit",
  "agentLocal.toolActivity.actions.skill": "Skill",
  "agentLocal.toolActivity.actions.agent": "Agent",
  "agentLocal.toolActivity.actions.forecast": "Forecast",
  "agentLocal.toolActivity.actions.mcp": "MCP",
  "agentLocal.toolActivity.actions.list": "List",
  "agentLocal.toolActivity.actions.search": "Search",
  "agentLocal.toolActivity.actions.run": "Run",
}[key] ?? key)) as TFunction;

describe("toolDisplayInfo", () => {
  it("raccourcit les chemins depuis la racine projet", () => {
    const path = "/Users/kevinh/Projects/systeme-pulse/src/App.tsx";
    expect(shortenPath(path, "/Users/kevinh/Projects/systeme-pulse")).toBe("systeme-pulse/src/App.tsx");
  });

  it("affiche une création avec stats + lignes + icône", () => {
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
      icon: "FilePlus",
    });
  });

  it("affiche une modification avec stats old/new + icône", () => {
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
      icon: "Pencil",
    });
  });

  it("ne change pas web_search", () => {
    expect(toolDisplayInfo({ name: "web_search", summary: "tauri docs" }, undefined, t)).toEqual({
      label: "web_search",
      summary: "tauri docs",
      icon: "Globe",
    });
  });

  it("garde bash comme nom de tool affiché", () => {
    expect(toolDisplayInfo({ name: "bash", summary: "npm test" }, undefined, t)).toEqual({
      label: "bash",
      summary: "npm test",
      icon: "TerminalWindow",
    });
  });

  it("affiche les tools spécialisés avec des noms explicites", () => {
    expect(toolDisplayInfo({ name: "load_skill", summary: "context7-docs" }, undefined, t).label).toBe("Skill");
    expect(toolDisplayInfo({ name: "delegate_task", summary: "audit" }, undefined, t).label).toBe("Agent");
    expect(toolDisplayInfo({ name: "forecast", summary: "sales" }, undefined, t).label).toBe("Read");
    expect(toolDisplayInfo({ name: "forecast_models", summary: "models" }, undefined, t).label).toBe("Read");
    expect(toolDisplayInfo({ name: "forecast_read", summary: "analysis" }, undefined, t).label).toBe("Read");
    expect(toolDisplayInfo({ name: "forecast_analyze", summary: "scenario" }, undefined, t).label).toBe("Forecast");
    expect(toolDisplayInfo({ name: "search_mcp_tools", summary: "linear" }, undefined, t).label).toBe("MCP");
  });

  it("tronque les commandes bash longues sur une seule ligne", () => {
    const command = `${"a".repeat(110)}\necho done`;
    expect(toolDisplayInfo({ name: "bash", summary: command }, undefined, t)).toEqual({
      label: "bash",
      summary: `${"a".repeat(96)}...`,
      icon: "TerminalWindow",
    });
  });

  it("associe la bonne icône Phosphor à chaque type d'outil", () => {
    const cases: Array<[string, string]> = [
      ["read_file", "BookOpenText"],
      ["read_spreadsheet", "FileText"],
      ["read_document", "FileText"],
      ["read_image", "Image"],
      ["write_file", "FilePlus"],
      ["write_spreadsheet", "FilePlus"],
      ["write_document", "FilePlus"],
      ["edit_file", "Pencil"],
      ["process_image", "Pencil"],
      ["bash", "TerminalWindow"],
      ["web_search", "Globe"],
      ["web_fetch", "Link"],
      ["list_dir", "FolderOpen"],
      ["grep", "MagnifyingGlass"],
      ["glob", "MagnifyingGlass"],
      ["create_branch", "GitBranch"],
      ["checkout_branch", "GitBranch"],
      ["load_skill", "Sparkle"],
      ["delegate_task", "Users"],
      ["forecast", "ChartLineUp"],
      ["forecast_analyze", "ChartLineUp"],
      ["search_mcp_tools", "Plugs"],
    ];
    for (const [name, expectedIcon] of cases) {
      const info = toolDisplayInfo({ name, summary: "x" }, undefined, t);
      expect(info.icon).toBe(expectedIcon);
    }
  });

  it("retourne l'icône Wrench par défaut pour un outil inconnu", () => {
    expect(toolDisplayInfo({ name: "outil_inconnu", summary: "x" }, undefined, t).icon).toBe("Wrench");
  });
});

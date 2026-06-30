import { describe, expect, it } from "vitest";
import { buildContextUsageBreakdown, CONTEXT_USAGE_KEYS } from "../context-usage-breakdown";
import type { AgentMessage } from "@/types/agent";

function msg(overrides: Partial<AgentMessage>): AgentMessage {
  return {
    id: crypto.randomUUID(),
    role: "user",
    content: "",
    files: [],
    timestamp: new Date().toISOString(),
    ...overrides,
  };
}

function item(tokens: ReturnType<typeof buildContextUsageBreakdown>["items"], key: string) {
  return tokens.find((entry) => entry.key === key)?.tokens ?? 0;
}

describe("context-usage-breakdown", () => {
  it("classe les messages simples dans Messages", () => {
    const breakdown = buildContextUsageBreakdown([msg({ content: "a".repeat(400) })]);

    expect(item(breakdown.items, "messages")).toBe(100);
    expect(breakdown.used).toBe(100);
  });

  it("compte le thinking dans Messages", () => {
    const breakdown = buildContextUsageBreakdown([
      msg({ content: "a".repeat(400), thinking: "b".repeat(400) }),
    ]);

    expect(item(breakdown.items, "messages")).toBe(200);
  });

  it("sépare les outils système et MCP", () => {
    const breakdown = buildContextUsageBreakdown([
      msg({
        role: "assistant",
        tool_activities: [
          { name: "bash", summary: "npm test", result: "ok" },
          { name: "mcp_fetch", summary: "docs", result: "ok" },
        ],
      }),
    ]);

    expect(item(breakdown.items, "systemTools")).toBeGreaterThan(0);
    expect(item(breakdown.items, "mcpConnectors")).toBeGreaterThan(0);
  });

  it("classe les skills chargés dans Skills", () => {
    const breakdown = buildContextUsageBreakdown([
      msg({ skill_names: ["frontend-skill"] }),
    ], { skillContextTokens: 12 });

    expect(item(breakdown.items, "skills")).toBeGreaterThan(12);
  });

  it("garde le system prompt isolé", () => {
    const breakdown = buildContextUsageBreakdown([], { systemPromptTokens: 42 });

    expect(item(breakdown.items, "systemPrompt")).toBe(42);
    expect(item(breakdown.items, "metaContext")).toBe(0);
  });

  it("range le delta observé dans Meta contexte", () => {
    const breakdown = buildContextUsageBreakdown([
      msg({ content: "a".repeat(400) }),
    ], { observedUsed: 180 });

    expect(item(breakdown.items, "messages")).toBe(100);
    expect(item(breakdown.items, "metaContext")).toBe(80);
  });

  it("garde un total cohérent avec les catégories", () => {
    const breakdown = buildContextUsageBreakdown([
      msg({ content: "a".repeat(400), skill_names: ["skill"] }),
      msg({ role: "tool", tool_name: "grep", content: "x".repeat(80) }),
    ], { systemPromptTokens: 10, metaContextTokens: 20 });
    const total = breakdown.items.reduce((sum, entry) => sum + entry.tokens, 0);

    expect(breakdown.items.map((entry) => entry.key)).toEqual(CONTEXT_USAGE_KEYS);
    expect(total).toBe(breakdown.used);
  });
});

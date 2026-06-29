import { describe, expect, it } from "vitest";
import { estimateAgentMessagesTokens, textUnits } from "../agent-token-estimate";
import type { AgentMessage } from "@/types/agent";

function msg(content: string): AgentMessage {
  return {
    id: "m1",
    role: "user",
    content,
    files: [],
    timestamp: new Date().toISOString(),
    tokens: 0,
  };
}

describe("agent-token-estimate", () => {
  it("garde le ratio ASCII historique", () => {
    expect(estimateAgentMessagesTokens([msg("a".repeat(400))])).toBe(100);
  });

  it("compte les accents comme non ASCII", () => {
    expect(estimateAgentMessagesTokens([msg("éé")])).toBe(1);
  });

  it("compte CJK et hangul avec prudence", () => {
    expect(estimateAgentMessagesTokens([msg("你".repeat(1000))])).toBe(1250);
    expect(estimateAgentMessagesTokens([msg("こ".repeat(1000))])).toBe(1250);
    expect(estimateAgentMessagesTokens([msg("한".repeat(1000))])).toBe(1250);
  });

  it("compte les emoji comme larges", () => {
    expect(textUnits("🎉")).toBe(5);
    expect(estimateAgentMessagesTokens([msg("🎉")])).toBe(2);
  });
});

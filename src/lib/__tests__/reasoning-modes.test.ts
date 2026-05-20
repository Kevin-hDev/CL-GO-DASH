import { describe, expect, it } from "vitest";
import {
  normalizeReasoningMode,
  reasoningModeOptions,
  type ReasoningMode,
} from "@/lib/reasoning-modes";
import type { AvailableModel } from "@/hooks/use-available-models";

function model(provider: string, id: string): AvailableModel {
  return {
    id,
    provider_id: provider,
    provider_name: provider,
    is_local: provider === "ollama",
    supports_tools: false,
    supports_thinking: true,
  };
}

function modes(provider: string, id: string): ReasoningMode[] {
  return reasoningModeOptions(model(provider, id)).map((option) => option.mode);
}

describe("reasoning modes", () => {
  it("n'affiche pas OFF pour Codex OAuth", () => {
    expect(modes("codex-oauth", "gpt-5.5")).toEqual(["low", "medium", "high", "xhigh"]);
  });

  it("affiche les niveaux Ollama GPT-OSS sans OFF", () => {
    expect(modes("ollama", "gpt-oss:20b")).toEqual(["low", "medium", "high"]);
  });

  it("normalise un mode invalide vers la valeur sûre", () => {
    const options = reasoningModeOptions(model("codex-oauth", "gpt-5.5"));
    expect(normalizeReasoningMode("off", options)).toBe("medium");
  });
});

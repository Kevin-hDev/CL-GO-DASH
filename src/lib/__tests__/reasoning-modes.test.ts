import { describe, expect, it } from "vitest";
import {
  normalizeReasoningMode,
  reasoningModeOptions,
  type ReasoningMode,
} from "@/lib/reasoning-modes";
import type { AvailableModel } from "@/hooks/use-available-models";

function model(provider: string, id: string, reasoningModes?: ReasoningMode[]): AvailableModel {
  return {
    id,
    provider_id: provider,
    provider_name: provider,
    is_local: provider === "ollama",
    supports_tools: false,
    supports_thinking: true,
    reasoning_modes: reasoningModes,
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

  it("n'affiche pas OFF pour les modèles X.ai reasoning dédiés", () => {
    expect(modes("xai", "grok-4-1-fast-reasoning")).toEqual(["low", "medium", "high", "xhigh"]);
    expect(modes("xai", "grok-4.20-multi-agent-beta-0309")).toEqual(["low", "medium", "high", "xhigh"]);
  });

  it("adapte Groq selon la famille reasoning", () => {
    expect(modes("groq", "openai/gpt-oss-20b")).toEqual(["low", "medium", "high"]);
    expect(modes("groq", "qwen/qwen3-32b")).toEqual(["off", "auto"]);
    expect(modes("groq", "openai/gpt-oss-safeguard-20b")).toEqual(["auto"]);
  });

  it("adapte DeepSeek, Mistral et Moonshot", () => {
    expect(modes("deepseek", "deepseek-v4-pro")).toEqual(["off", "high", "xhigh"]);
    expect(modes("mistral", "mistral-small-latest")).toEqual(["off", "high"]);
    expect(modes("mistral", "magistral-medium-latest")).toEqual(["auto"]);
    expect(modes("mistral", "mistral-small-2506")).toEqual([]);
    expect(modes("moonshot", "kimi-k2.5")).toEqual(["off", "auto"]);
    expect(modes("moonshot", "kimi-k2-thinking")).toEqual(["auto"]);
  });

  it("affiche OFF/AUTO pour Z.ai quand le modèle supporte le thinking", () => {
    expect(modes("zai", "glm-5")).toEqual(["off", "auto"]);
    expect(modes("zai", "glm-4.6")).toEqual(["off", "auto"]);
    expect(modes("zai", "glm-4.5-flash")).toEqual(["off", "auto"]);
  });

  it("utilise les modes dynamiques exposés par OpenRouter", () => {
    const openrouter = model("openrouter", "z-ai/glm-5.1", ["off", "auto", "low", "medium", "high", "xhigh"]);
    expect(reasoningModeOptions(openrouter).map((option) => option.mode)).toEqual([
      "off",
      "auto",
      "low",
      "medium",
      "high",
      "xhigh",
    ]);
  });

  it("normalise un mode invalide vers la valeur sûre", () => {
    const options = reasoningModeOptions(model("codex-oauth", "gpt-5.5"));
    expect(normalizeReasoningMode("off", options)).toBe("medium");
  });
});

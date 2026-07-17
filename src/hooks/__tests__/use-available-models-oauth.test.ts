import { describe, expect, it } from "vitest";
import { mapOAuthModels, withoutInteractiveOnlyModels } from "../use-available-models";

describe("OAuth models", () => {
  it("utilise des ids et libellés distincts des providers API", () => {
    const groups = mapOAuthModels([
      { id: "kimi-for-coding", provider_id: "moonshot", display_name: "Kimi", supports_tools: true, supports_vision: true, supports_thinking: true, interactive_only: true },
      { id: "grok-4.3", provider_id: "xai", display_name: "Grok 4.3", supports_tools: true, supports_vision: true, supports_thinking: true, interactive_only: true },
      { id: "gpt-5.6-sol", provider_id: "openai", display_name: "gpt-5.6-sol", supports_tools: true, supports_vision: true, supports_thinking: true, interactive_only: false },
    ]);

    expect(groups.get("moonshot-oauth")?.[0].provider_name).toBe("Moonshot AI · OAuth");
    expect(groups.get("xai-oauth")?.[0].provider_name).toBe("xAI · OAuth");
    expect(groups.get("codex-oauth")?.[0].provider_name).toBe("OpenAI · OAuth");
    expect(groups.has("moonshot")).toBe(false);
    expect(groups.has("xai")).toBe(false);
    expect(groups.get("moonshot-oauth")?.[0].interactive_only).toBe(true);

    const automated = withoutInteractiveOnlyModels(groups);
    expect(automated.has("moonshot-oauth")).toBe(false);
    expect(automated.has("xai-oauth")).toBe(false);
    expect(automated.has("codex-oauth")).toBe(true);
  });
});

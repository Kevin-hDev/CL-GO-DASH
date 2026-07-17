import { describe, expect, it } from "vitest";
import { mapOAuthModels } from "../use-available-models";

describe("OAuth models", () => {
  it("utilise des ids et libellés distincts des providers API", () => {
    const groups = mapOAuthModels([
      { id: "kimi-code", provider_id: "moonshot", display_name: "Kimi Code", supports_tools: true, supports_vision: false, supports_thinking: true },
      { id: "grok-build", provider_id: "xai", display_name: "Grok Build", supports_tools: true, supports_vision: false, supports_thinking: true },
      { id: "gpt-5.6-sol", provider_id: "openai", display_name: "gpt-5.6-sol", supports_tools: true, supports_vision: true, supports_thinking: true },
    ]);

    expect(groups.get("moonshot-oauth")?.[0].provider_name).toBe("Moonshot AI · OAuth");
    expect(groups.get("xai-oauth")?.[0].provider_name).toBe("xAI · OAuth");
    expect(groups.get("codex-oauth")?.[0].provider_name).toBe("OpenAI · OAuth");
    expect(groups.has("moonshot")).toBe(false);
    expect(groups.has("xai")).toBe(false);
  });
});

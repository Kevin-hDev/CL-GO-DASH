import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { OnboardingApi } from "../onboarding-api";
import type { ProviderSpec } from "@/types/api";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-shell", () => ({
  open: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: Record<string, string>) =>
      values?.name ? `${key}:${values.name}` : key,
    i18n: { language: "fr" },
  }),
}));

vi.mock("@/lib/provider-icons", () => ({
  ProviderIcon: ({ displayName }: { displayName: string }) => (
    <span data-testid="provider-icon">{displayName}</span>
  ),
}));

vi.mock("@/lib/toast-emitter", () => ({
  showToast: vi.fn(),
}));

function provider(id: string, category: ProviderSpec["category"]): ProviderSpec {
  return {
    id,
    category,
    display_name: id,
    signup_url: "https://example.com",
    free_tier_label: "",
    short_description: `${id} fr`,
    short_description_en: `${id} en`,
  };
}

describe("OnboardingApi", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_llm_providers_catalog") {
        return Promise.resolve([
          provider("openai", "llm"),
          provider("mistral", "llm"),
          provider("brave", "search"),
        ]);
      }
      if (command === "list_configured_providers") {
        return Promise.resolve([]);
      }
      return Promise.resolve();
    });
  });

  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("affiche uniquement les providers LLM", async () => {
    render(<OnboardingApi onComplete={vi.fn()} />);

    await waitFor(() => expect(screen.getAllByText("openai").length).toBeGreaterThan(0));

    expect(screen.queryByText("brave")).toBeNull();
  });

  it("affiche les providers deja configures", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_llm_providers_catalog") {
        return Promise.resolve([
          provider("openai", "llm"),
          provider("mistral", "llm"),
        ]);
      }
      if (command === "list_configured_providers") {
        return Promise.resolve(["mistral"]);
      }
      return Promise.resolve();
    });

    render(<OnboardingApi onComplete={vi.fn()} />);

    await waitFor(() => expect(screen.getAllByText("mistral").length).toBeGreaterThan(0));
    expect(screen.getByText("apiKeys.details.connected")).toBeTruthy();
  });

  it("teste puis enregistre la cle sans quitter la page", async () => {
    const onComplete = vi.fn();
    render(<OnboardingApi onComplete={onComplete} />);

    await waitFor(() => expect(screen.getAllByText("openai").length).toBeGreaterThan(0));
    fireEvent.change(screen.getByLabelText("onboarding.api.keyLabel:openai"), {
      target: { value: "sk-test" },
    });
    fireEvent.click(screen.getByText("apiKeys.dialog.addAndTest"));

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("test_api_key_with_value", {
      provider: "openai",
      key: "sk-test",
    }));
    expect(invoke).toHaveBeenCalledWith("set_api_key", {
      provider: "openai",
      key: "sk-test",
    });
    expect(await screen.findByText("apiKeys.dialog.testOk")).toBeTruthy();
    expect(screen.getByText("apiKeys.details.connected")).toBeTruthy();
    expect(onComplete).not.toHaveBeenCalled();
  });

  it("remasque la cle quand l'utilisateur change de provider", async () => {
    render(<OnboardingApi onComplete={vi.fn()} />);

    await waitFor(() => expect(screen.getAllByText("openai").length).toBeGreaterThan(0));
    const openAiInput = screen.getByLabelText("onboarding.api.keyLabel:openai");
    fireEvent.change(openAiInput, { target: { value: "sk-test" } });
    fireEvent.click(screen.getByRole("button", { name: "apiKeys.dialog.showKey" }));
    expect(openAiInput.getAttribute("type")).toBe("text");

    fireEvent.click(screen.getByRole("button", { name: /mistral/ }));

    const mistralInput = screen.getByLabelText("onboarding.api.keyLabel:mistral");
    expect(mistralInput.getAttribute("type")).toBe("password");
  });

  it("continue seulement quand l'utilisateur passe l'etape", async () => {
    const onComplete = vi.fn();
    render(<OnboardingApi onComplete={onComplete} />);

    await waitFor(() => expect(screen.getAllByText("openai").length).toBeGreaterThan(0));
    fireEvent.click(screen.getByText("onboarding.common.skip"));

    await waitFor(() => expect(onComplete).toHaveBeenCalled());
  });

  it("permet de passer sans enregistrer de cle", async () => {
    const onComplete = vi.fn();
    render(<OnboardingApi onComplete={onComplete} />);

    fireEvent.click(screen.getByText("onboarding.common.skip"));

    await waitFor(() => expect(onComplete).toHaveBeenCalled());
    expect(invoke).not.toHaveBeenCalledWith("set_api_key", expect.anything());
  });
});

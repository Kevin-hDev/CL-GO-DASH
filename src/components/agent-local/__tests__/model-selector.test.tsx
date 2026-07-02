import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ModelSelector } from "../model-selector";
import type { AvailableModel } from "@/hooks/use-available-models";

let groups = new Map<string, AvailableModel[]>();

vi.mock("@/hooks/use-available-models", () => ({
  useAvailableModels: () => ({ groups }),
}));

vi.mock("@/hooks/use-favorite-models", () => ({
  useFavoriteModels: () => ({
    favorites: [],
    isFavorite: () => false,
    toggle: vi.fn(),
  }),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const labels: Record<string, string> = {
        "agentLocal.selectModel": "Select model",
        "agentLocal.reasoningAuto": "Activée",
        "agentLocal.reasoningHigh": "Forte",
      };
      return labels[key] ?? key;
    },
  }),
}));

afterEach(() => {
  cleanup();
  groups = new Map();
});

function model(overrides: Partial<AvailableModel>): AvailableModel {
  return {
    id: "llama3",
    provider_id: "ollama",
    provider_name: "Ollama",
    is_local: true,
    supports_tools: false,
    supports_thinking: true,
    ...overrides,
  };
}

describe("ModelSelector", () => {
  it("remplace le libellé Activée par l'icône cerveau pour un toggle simple", () => {
    groups = new Map([["ollama", [model({ id: "llama3" })]]]);

    const { container } = render(
      <ModelSelector
        selectedModel="llama3"
        selectedProvider="ollama"
        reasoningMode="auto"
        onSelect={vi.fn()}
        onReasoningModeChange={vi.fn()}
      />,
    );

    expect(screen.getByText("llama3")).toBeTruthy();
    expect(screen.queryByText("Activée")).toBeNull();
    expect(container.querySelector(".ms-trigger-reasoning-icon")).toBeTruthy();
  });

  it("garde le libellé d'effort et ajoute l'icône cerveau", () => {
    groups = new Map([["openai", [model({
      id: "gpt-5",
      provider_id: "openai",
      provider_name: "OpenAI",
      is_local: false,
    })]]]);

    const { container } = render(
      <ModelSelector
        selectedModel="gpt-5"
        selectedProvider="openai"
        reasoningMode="high"
        onSelect={vi.fn()}
        onReasoningModeChange={vi.fn()}
      />,
    );

    expect(screen.getByText("gpt-5")).toBeTruthy();
    expect(screen.getByText("Forte")).toBeTruthy();
    expect(container.querySelector(".ms-trigger-reasoning-icon")).toBeTruthy();
  });

  it("ouvre la liste des modèles dans un portail global", () => {
    groups = new Map([["ollama", [model({ id: "llama3" })]]]);

    const { container } = render(
      <div data-testid="host">
        <ModelSelector
          selectedModel="llama3"
          selectedProvider="ollama"
          reasoningMode="auto"
          onSelect={vi.fn()}
          onReasoningModeChange={vi.fn()}
        />
      </div>,
    );

    fireEvent.click(screen.getByText("llama3"));

    expect(document.body.querySelector(".ms-dropdown")).toBeTruthy();
    expect(container.querySelector(".ms-dropdown")).toBeNull();
  });
});

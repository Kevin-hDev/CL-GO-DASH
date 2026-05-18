/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { StrictMode } from "react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import type { SettingsSubTab } from "@/types/navigation";
import {
  CHILD_COMMANDS,
  invokedCommands,
  resetSettingsTestEnvironment,
  SettingsHarness,
} from "../test-utils/settings-tab-test-setup";

describe("SettingsTab slots", () => {
  afterEach(() => cleanup());

  beforeEach(() => {
    resetSettingsTestEnvironment();
  });

  it("ne monte pas les sous-onglets sur les pages principales", async () => {
    render(<StrictMode><SettingsHarness /></StrictMode>);

    await screen.findAllByText("settings.tabs.general");
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(invokedCommands().some((cmd) => CHILD_COMMANDS.has(cmd))).toBe(false);
  });

  it.each([
    ["ollama", "settings.tabs.ollama", "llama3.2:latest"],
    ["connectors", "settings.tabs.connectors", "Canva"],
    ["channels", "settings.tabs.channels", "test-telegram"],
    ["api-keys", "settings.tabs.apiKeys", "Groq"],
    ["forecast", "forecast.title", "Chronos Bolt Small"],
  ] as Array<[SettingsSubTab, string, string]>)("ouvre %s sans crash ni boucle", async (_subTab, label, expectedContent) => {
    render(<StrictMode><SettingsHarness /></StrictMode>);
    const [item] = await screen.findAllByText(label);
    fireEvent.click(item);

    await waitFor(() => {
      const active = screen.getAllByText(label)
        .some((element) => element.getAttribute("data-nav-active") === "true");
      expect(active).toBe(true);
    });

    await waitFor(() => expect(screen.getAllByText(expectedContent).length).toBeGreaterThan(0));
  });
});

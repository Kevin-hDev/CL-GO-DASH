/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { UpdateNotifications } from "../update-notifications";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    i18n: { language: "fr" },
    t: (key: string, opts?: Record<string, string>) => {
      if (key === "updates.version") return `Version ${opts?.version ?? ""}`;
      if (key === "updates.releaseNotesTitle") return `Notes ${opts?.version ?? ""}`;
      return key;
    },
  }),
}));

const baseProps = {
  isOpen: true,
  onClose: vi.fn(),
  appUpdate: null,
  ollamaBinaryUpdate: null,
  ollamaUpdates: [],
  pulling: null,
  ollamaBinaryUpdating: false,
  ollamaBinaryPercent: 0,
  appDownloading: false,
  appPercent: 0,
  onPullModel: vi.fn(),
  onDownloadApp: vi.fn(),
  onUpdateOllamaBinary: vi.fn(),
  anchorLeft: 0,
};

describe("UpdateNotifications", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("garde les updates Ollama compactes", () => {
    render(
      <UpdateNotifications
        {...baseProps}
        ollamaUpdates={[{ fullName: "llama3:latest", family: "llama3", tag: "latest" }]}
      />,
    );

    expect(screen.getByText("llama3:latest")).toBeTruthy();
    expect(screen.queryByLabelText("updates.showDetails")).toBeNull();
  });

  it("déplie et replie les notes de l'update app", () => {
    render(
      <UpdateNotifications
        {...baseProps}
        appUpdate={{
          version: "0.9.4",
          assetUrl: "https://example.invalid/app.dmg",
          notesByLocale: {
            en: ["Context details."],
            fr: ["Détails du contexte."],
          },
        }}
      />,
    );

    const toggle = screen.getByLabelText("updates.showDetails");
    expect(toggle).toHaveAttribute("aria-expanded", "false");

    fireEvent.click(toggle);
    expect(screen.getByLabelText("updates.hideDetails")).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByText("Détails du contexte.")).toBeTruthy();

    fireEvent.click(screen.getByLabelText("updates.hideDetails"));
    expect(screen.getByLabelText("updates.showDetails")).toHaveAttribute("aria-expanded", "false");
  });

  it("masque la flèche si l'update app n'a pas de notes", () => {
    render(
      <UpdateNotifications
        {...baseProps}
        appUpdate={{
          version: "0.9.4",
          assetUrl: "https://example.invalid/app.dmg",
          notesByLocale: null,
        }}
      />,
    );

    expect(screen.getByText("CL-GO")).toBeTruthy();
    expect(screen.queryByLabelText("updates.showDetails")).toBeNull();
  });
});

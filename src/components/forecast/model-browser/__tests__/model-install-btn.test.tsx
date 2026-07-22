/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ModelDownloadState } from "@/hooks/use-model-downloads";
import { ModelInstallBtn } from "../model-install-btn";

const startDownload = vi.fn();
const cancelDownload = vi.fn();
const invoke = vi.hoisted(() => vi.fn());
type InstallHookValue = {
  activeDownload: Partial<ModelDownloadState> | null;
  startDownload: typeof startDownload;
  cancelDownload: typeof cancelDownload;
  downloads: Partial<ModelDownloadState>[];
};

const mockedUseModelDownloads = vi.fn<() => InstallHookValue>();

vi.mock("@/hooks/use-model-downloads", () => ({
  useModelDownloads: () => mockedUseModelDownloads(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke }));

vi.mock("@/components/ui/icons", () => ({
  Check: () => <span data-testid="check" />,
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("ModelInstallBtn", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  beforeEach(() => {
    startDownload.mockResolvedValue(undefined);
    cancelDownload.mockResolvedValue(undefined);
    invoke.mockResolvedValue(undefined);
  });

  it("desactive le bouton si un autre telechargement tourne", () => {
    mockedUseModelDownloads.mockReturnValue({
      activeDownload: { id: "other", kind: "ollama", modelId: "llama3" },
      startDownload,
      cancelDownload,
      downloads: [],
    });

    render(
      <ModelInstallBtn
        modelId="chronos-tiny"
        installed={false}
        runtimeReady={false}
        onDone={vi.fn()}
      />,
    );

    expect(screen.getByRole("button").hasAttribute("disabled")).toBe(true);
    expect(screen.getByText("modelDownloads.busy")).toBeTruthy();
  });

  it("garde la progression visible pour son propre modele", () => {
    mockedUseModelDownloads.mockReturnValue({
      activeDownload: {
        id: "forecast-1",
        kind: "forecast",
        modelId: "chronos-tiny",
        status: "running",
        phase: "preparing-runtime",
        percent: 58,
      },
      startDownload,
      cancelDownload,
      downloads: [],
    });

    render(
      <ModelInstallBtn
        modelId="chronos-tiny"
        installed={false}
        runtimeReady={false}
        onDone={vi.fn()}
      />,
    );

    expect(screen.getByText("modelDownloads.phases.preparing-runtime")).toBeTruthy();
    expect(screen.getByText("58%")).toBeTruthy();
    fireEvent.click(screen.getByText("forecast.models.cancel"));
    expect(cancelDownload).toHaveBeenCalledWith("forecast-1");
  });

  it("demarre le telechargement global", () => {
    mockedUseModelDownloads.mockReturnValue({
      activeDownload: null,
      startDownload,
      cancelDownload,
      downloads: [],
    });

    render(
      <ModelInstallBtn
        modelId="chronos-tiny"
        installed={false}
        runtimeReady={false}
        onDone={vi.fn()}
      />,
    );
    fireEvent.click(screen.getByRole("button"));

    expect(startDownload).toHaveBeenCalledWith({
      kind: "forecast",
      modelId: "chronos-tiny",
    });
  });

  it("propose de preparer un modele dont le moteur manque", () => {
    mockedUseModelDownloads.mockReturnValue({
      activeDownload: null,
      startDownload,
      cancelDownload,
      downloads: [],
    });

    render(
      <ModelInstallBtn
        modelId="moirai-small"
        installed
        runtimeReady={false}
        onDone={vi.fn()}
      />,
    );
    fireEvent.click(screen.getByText("forecast.models.prepare"));

    expect(startDownload).toHaveBeenCalledWith({ kind: "forecast", modelId: "moirai-small" });
  });

  it("confirme la desinstallation depuis la fiche modele", async () => {
    const onDone = vi.fn();
    mockedUseModelDownloads.mockReturnValue({
      activeDownload: null,
      startDownload,
      cancelDownload,
      downloads: [],
    });

    render(
      <ModelInstallBtn
        modelId="moirai-small"
        installed
        runtimeReady
        allowUninstall
        onDone={onDone}
      />,
    );
    fireEvent.click(screen.getByText("forecast.models.uninstall"));
    fireEvent.click(screen.getByText("forecast.models.confirmUninstall"));

    await waitFor(() => expect(invoke).toHaveBeenCalledWith(
      "uninstall_forecast_model",
      { name: "moirai-small" },
    ));
    expect(onDone).toHaveBeenCalledOnce();
  });
});

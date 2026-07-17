/* @vitest-environment jsdom */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { StrictMode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { OAuthLoginProgress, OAuthProviderStatus } from "@/types/oauth-provider";
import { OAuthProviderLoginDialog } from "../oauth-provider-login-dialog";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(() => Promise.resolve()) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));
vi.mock("@tauri-apps/plugin-shell", () => ({ open: vi.fn(() => Promise.resolve()) }));

const moonshot: OAuthProviderStatus = {
  id: "moonshot",
  display_name: "Moonshot AI",
  connected: false,
  account: null,
  client_state: "ready",
  install_url: "https://www.kimi.com/code/docs/en/",
};

describe("OAuthProviderLoginDialog", () => {
  let progress: ((event: { payload: OAuthLoginProgress }) => void) | undefined;
  let clipboardWrite: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.mocked(invoke).mockClear().mockResolvedValue(undefined);
    vi.mocked(listen).mockImplementation((_event, handler) => {
      progress = handler as typeof progress;
      return Promise.resolve(() => undefined);
    });
    clipboardWrite = vi.fn(() => Promise.resolve());
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText: clipboardWrite },
    });
  });

  it("lance la connexion, affiche l'attente et permet de relancer", async () => {
    render(<StrictMode><OAuthProviderLoginDialog provider={moonshot} onClose={vi.fn()} onConnected={vi.fn()} /></StrictMode>);

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("start_oauth_provider_login", { providerId: "moonshot" }));
    expect(vi.mocked(invoke).mock.calls.filter(([command]) => command === "start_oauth_provider_login")).toHaveLength(1);
    progress?.({ payload: { provider_id: "moonshot", stage: "waiting" } });
    expect(await screen.findByText("connectors.oauth.message")).toBeTruthy();

    fireEvent.click(screen.getByText("connectors.oauth.retry"));
    await waitFor(() => {
      expect(vi.mocked(invoke).mock.calls.filter(([command]) => command === "start_oauth_provider_login")).toHaveLength(2);
    });
  });

  it("ne valide la connexion qu'après le signal de succès", async () => {
    const onConnected = vi.fn();
    render(<OAuthProviderLoginDialog provider={moonshot} onClose={vi.fn()} onConnected={onConnected} />);
    await waitFor(() => expect(progress).toBeTypeOf("function"));
    expect(onConnected).not.toHaveBeenCalled();

    progress?.({ payload: { provider_id: "moonshot", stage: "success" } });
    await waitFor(() => expect(onConnected).toHaveBeenCalledTimes(1));
  });

  it("explique le client manquant sans présenter Installer comme action OAuth", async () => {
    render(<OAuthProviderLoginDialog provider={{ ...moonshot, id: "xai", display_name: "xAI", client_state: "missing" }} onClose={vi.fn()} onConnected={vi.fn()} />);

    expect(await screen.findByText("providers.oauth.clientRequired")).toBeTruthy();
    expect(screen.queryByText("providers.oauth.install")).toBeNull();
    expect(invoke).not.toHaveBeenCalledWith("start_oauth_provider_login", expect.anything());
  });

  it("affiche et copie le code que Grok demande de saisir dans le navigateur", async () => {
    render(<OAuthProviderLoginDialog provider={{ ...moonshot, id: "xai", display_name: "xAI" }} onClose={vi.fn()} onConnected={vi.fn()} />);
    await waitFor(() => expect(progress).toBeTypeOf("function"));

    progress?.({
      payload: {
        provider_id: "xai",
        stage: "verification",
        user_code: "45JE-V2VK",
        verification_url: "https://auth.x.ai/device",
      },
    });

    expect(await screen.findByText("45JE-V2VK")).toBeTruthy();
    fireEvent.click(screen.getByText("providers.oauth.copyCode"));
    expect(clipboardWrite).toHaveBeenCalledWith("45JE-V2VK");
    expect(screen.queryByRole("textbox")).toBeNull();
  });
});

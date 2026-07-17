import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-shell";
import { useTranslation } from "react-i18next";
import { ProviderIcon } from "@/lib/provider-icons";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { OAuthLoginProgress, OAuthProviderStatus } from "@/types/oauth-provider";
import "@/components/connectors/mcp-oauth-dialog.css";

type LoginState = "loading" | "waiting" | "success" | "error";

interface OAuthProviderLoginDialogProps {
  provider: OAuthProviderStatus;
  onClose: () => void;
  onConnected: () => void;
}

export function OAuthProviderLoginDialog({ provider, onClose, onConnected }: OAuthProviderLoginDialogProps) {
  const { t } = useTranslation();
  const [state, setState] = useState<LoginState>("loading");
  const [hint, setHint] = useState<string | null>(null);
  const mountedRef = useRef(true);
  const onConnectedRef = useRef(onConnected);

  useEffect(() => {
    onConnectedRef.current = onConnected;
  }, [onConnected]);

  const startFlow = useCallback(async () => {
    setHint(null);
    if (provider.client_state !== "ready") {
      setState("error");
      return;
    }
    setState("loading");
    try {
      await invoke("cancel_oauth_provider_login", { providerId: provider.id });
      if (!mountedRef.current) return;
      await invoke("start_oauth_provider_login", { providerId: provider.id });
    } catch {
      if (mountedRef.current) setState("error");
    }
  }, [provider.client_state, provider.id]);

  useEffect(() => {
    mountedRef.current = true;
    let active = true;
    let successTimer: ReturnType<typeof setTimeout> | undefined;
    const unlisten = listen<OAuthLoginProgress>("oauth-login-progress", ({ payload }) => {
      if (!mountedRef.current || payload.provider_id !== provider.id) return;
      if (payload.stage === "success") {
        setState("success");
        successTimer = setTimeout(() => onConnectedRef.current(), 600);
      } else if (payload.stage === "waiting" || payload.stage === "verification") {
        setHint(payload.hint ?? null);
        setState("waiting");
      } else {
        setState("error");
      }
    });
    void unlisten.then(() => { if (active) void startFlow(); });
    return () => {
      active = false;
      mountedRef.current = false;
      if (successTimer) clearTimeout(successTimer);
      cleanupTauriListener(unlisten);
    };
  }, [provider.id, startFlow]);

  const handleClose = () => {
    void invoke("cancel_oauth_provider_login", { providerId: provider.id });
    onClose();
  };
  const missingClient = provider.client_state !== "ready";
  const message = missingClient
    ? t(provider.client_state === "missing" ? "providers.oauth.clientRequired" : "providers.oauth.clientIncompatible")
    : hint ?? t(state === "waiting" ? "connectors.oauth.message" : state === "success" ? "providers.oauth.successMessage" : state === "error" ? "connectors.oauth.errorGeneric" : "connectors.oauth.discovering");

  return (
    <div className="wk-dialog-overlay" role="presentation" onClick={handleClose} onKeyDown={() => undefined}>
      {/* eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions -- dialog stop-propagation pattern */}
      <div className="wk-dialog mco-dialog" role="dialog" onClick={(event) => event.stopPropagation()} onKeyDown={() => undefined}>
        <div className="mco-icons">
          <div className="mco-icon-box"><ProviderIcon providerId={provider.id} displayName={provider.display_name} size={40} /></div>
          <div className="mco-connector-line" />
          <div className="mco-icon-box"><span className="mco-app-icon">CL</span></div>
        </div>
        <h3 className="mco-title">{t(state === "success" ? "connectors.oauth.successTitle" : "connectors.oauth.title")}</h3>
        <p className={`mco-message ${state === "error" ? "mco-error" : ""}`}>{message}</p>
        {!missingClient && (state === "waiting" || state === "error") && (
          <button type="button" className="mco-retry-link" onClick={() => void startFlow()}>{t("connectors.oauth.retry")}</button>
        )}
        {missingClient && (
          <button type="button" className="mco-retry-link" onClick={() => void open(provider.install_url)}>{t("providers.oauth.installationGuide")}</button>
        )}
        <div className="wk-dialog-footer mco-footer">
          <button type="button" className="wk-btn-secondary" onClick={handleClose}>{t("connectors.oauth.cancel")}</button>
        </div>
      </div>
    </div>
  );
}

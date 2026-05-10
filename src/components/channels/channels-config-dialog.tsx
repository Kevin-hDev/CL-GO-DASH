import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { X } from "@/components/ui/icons";
import type { ChannelType } from "@/types/channels";

interface ChannelsConfigDialogProps {
  channelId: ChannelType;
  onClose: () => void;
  onSaved: (accountId: string) => void;
}

type TestState =
  | { kind: "idle" }
  | { kind: "testing" }
  | { kind: "ok" }
  | { kind: "error"; message: string };

export function ChannelsConfigDialog({ channelId, onClose, onSaved }: ChannelsConfigDialogProps) {
  const { t } = useTranslation();
  const [accountId, setAccountId] = useState("");
  const [token, setToken] = useState("");
  const [testState, setTestState] = useState<TestState>({ kind: "idle" });
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key.startsWith("Esc")) {
        e.preventDefault();
        onClose();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!accountId.trim() || !token.trim()) return;
    setSubmitting(true);
    setTestState({ kind: "testing" });
    try {
      await invoke("gateway_set_token", {
        channelId,
        accountId: accountId.trim(),
        token: token.trim(),
      });
      const verified = await invoke<boolean>("gateway_has_token", {
        channelId,
        accountId: accountId.trim(),
      });
      if (!verified) {
        setTestState({ kind: "error", message: t("channels.detail.testFailed") });
        return;
      }
      setTestState({ kind: "ok" });
      setTimeout(() => onSaved(accountId.trim()), 500);
    } catch {
      setTestState({ kind: "error", message: t("channels.detail.testFailed") });
    } finally {
      setSubmitting(false);
    }
  };

  const channelName = channelId.charAt(0).toUpperCase() + channelId.slice(1);

  return (
    <div className="wk-dialog-overlay" role="presentation" onClick={onClose} onKeyDown={() => {}}>
      <div className="wk-dialog" role="presentation" onClick={(e) => e.stopPropagation()} onKeyDown={() => {}}>
        <header className="wk-dialog-header">
          <span>{t("channels.config.addTitle", { name: channelName })}</span>
          <button type="button" className="wk-dialog-close" onClick={onClose}>
            <X size={16} />
          </button>
        </header>

        <form className="wk-form" onSubmit={(e) => void handleSubmit(e)}>
          <div className="wk-form-field">
            <label className="wk-form-label">{t("channels.detail.accountId")}</label>
            <input
              type="text"
              className="wk-input"
              value={accountId}
              onChange={(e) => setAccountId(e.target.value)}
              placeholder="my-bot"
              autoFocus
            />
          </div>

          <div className="wk-form-field">
            <label className="wk-form-label">{t("channels.detail.token")}</label>
            <input
              type="password"
              className="wk-input"
              value={token}
              onChange={(e) => setToken(e.target.value)}
              placeholder={t("channels.detail.tokenPlaceholder")}
            />
          </div>

          <div className="wk-form-field">
            <label className="wk-form-label">{t("channels.config.description")}</label>
            <div className="wk-input" style={{ cursor: "default", opacity: 0.7 }}>
              {t(`channels.browse.${channelId}Desc`)}
            </div>
          </div>

          {testState.kind === "testing" && (
            <div className="ak-test-result loading">{t("channels.config.testing")}</div>
          )}
          {testState.kind === "ok" && (
            <div className="ak-test-result success">{t("channels.detail.testSuccess")}</div>
          )}
          {testState.kind === "error" && (
            <div className="ak-test-result error">{testState.message}</div>
          )}

          <footer className="wk-dialog-footer">
            <button type="button" className="wk-btn-secondary" onClick={onClose} disabled={submitting}>
              {t("channels.detail.cancel")}
            </button>
            <button
              type="submit"
              className="wk-btn-primary"
              disabled={submitting || !accountId.trim() || !token.trim()}
            >
              {t("channels.config.addAndTest")}
            </button>
          </footer>
        </form>
      </div>
    </div>
  );
}

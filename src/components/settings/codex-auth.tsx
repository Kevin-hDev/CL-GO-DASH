import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import "./codex-auth.css";

interface CodexStatus {
    logged_in: boolean;
    email: string | null;
}

export function CodexAuth() {
    const { t } = useTranslation();
    const [status, setStatus] = useState<CodexStatus | null>(null);
    const [loading, setLoading] = useState(false);

    const refresh = async () => {
        try {
            const s = await invoke<CodexStatus>("codex_status");
            setStatus(s);
        } catch {
            setStatus({ logged_in: false, email: null });
        }
    };

    useEffect(() => {
        refresh();
    }, []);

    const handleLogin = async () => {
        setLoading(true);
        try {
            await invoke<string>("codex_login");
            await refresh();
        } catch (e) {
            console.error("codex login:", e);
        } finally {
            setLoading(false);
        }
    };

    const handleLogout = async () => {
        try {
            await invoke("codex_logout");
            await refresh();
        } catch (e) {
            console.error("codex logout:", e);
        }
    };

    const loggedIn = status?.logged_in ?? false;

    return (
        <div className="cdx-auth">
            <div className="cdx-auth-status">
                <span
                    className={`cdx-auth-dot ${loggedIn ? "cdx-auth-dot--on" : "cdx-auth-dot--off"}`}
                />
                <span>
                    {loggedIn
                        ? t("codex.loggedIn")
                        : t("codex.loggedOut")}
                </span>
                {status?.email && (
                    <span className="cdx-auth-email">
                        ({status.email})
                    </span>
                )}
            </div>
            {loggedIn ? (
                <button className="ollama-btn" onClick={handleLogout}>
                    {t("codex.logout")}
                </button>
            ) : (
                <button
                    className="ollama-btn"
                    onClick={handleLogin}
                    disabled={loading}
                >
                    {loading
                        ? t("codex.loggingIn")
                        : t("codex.login")}
                </button>
            )}
        </div>
    );
}

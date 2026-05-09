import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";
import "./codex-auth.css";

interface CodexStatus {
    logged_in: boolean;
    email: string | null;
}

const EFFORT_LEVELS = [
    { value: "low", label: "codex.effortLow" },
    { value: "medium", label: "codex.effortMedium" },
    { value: "high", label: "codex.effortHigh" },
    { value: "xhigh", label: "codex.effortXhigh" },
] as const;

export function CodexAuth() {
    const { t } = useTranslation();
    const [status, setStatus] = useState<CodexStatus | null>(null);
    const [loading, setLoading] = useState(false);
    const [effort, setEffort] = useState("medium");

    const refresh = useCallback(async () => {
        try {
            const s = await invoke<CodexStatus>("codex_status");
            setStatus(s);
            const e = await invoke<string>("codex_get_effort");
            setEffort(e);
        } catch {
            setStatus({ logged_in: false, email: null });
        }
    }, []);

    useEffect(() => {
        // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
        void refresh();
    }, [refresh]);

    useEffect(() => {
        const unlisten = listen("codex-auth-changed", () => { void refresh(); });
        return () => { void unlisten.then((fn) => fn()); };
    }, [refresh]);

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

    const handleEffort = async (level: string) => {
        setEffort(level);
        await invoke("codex_set_effort", { level });
    };

    const loggedIn = status?.logged_in ?? false;

    return (
        <div className="cdx-auth">
            <div className="cdx-auth-status">
                <span
                    className={`cdx-auth-dot ${loggedIn ? "cdx-auth-dot--on" : "cdx-auth-dot--off"}`}
                />
                <span>
                    {loggedIn ? t("codex.loggedIn") : t("codex.loggedOut")}
                </span>
                {status?.email && (
                    <span className="cdx-auth-email">({status.email})</span>
                )}
            </div>
            {loggedIn ? (
                <>
                    <div className="cdx-auth-effort">
                        <span className="cdx-auth-effort-label">
                            {t("codex.effortLabel")}
                        </span>
                        <div className="cdx-auth-effort-btns">
                            {EFFORT_LEVELS.map((l) => (
                                <button
                                    key={l.value}
                                    className={`cdx-effort-btn${effort === l.value ? " cdx-effort-btn--active" : ""}`}
                                    onClick={() => void handleEffort(l.value)}
                                >
                                    {t(l.label)}
                                </button>
                            ))}
                        </div>
                    </div>
                    <button className="ollama-btn" onClick={() => void handleLogout()}>
                        {t("codex.logout")}
                    </button>
                </>
            ) : (
                <button
                    className="ollama-btn"
                    onClick={() => void handleLogin()}
                    disabled={loading}
                >
                    {loading ? t("codex.loggingIn") : t("codex.login")}
                </button>
            )}
        </div>
    );
}

import { useState, useEffect, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { ThemedIcon } from "@/components/ui/themed-icon";
import type { AppUpdate, OllamaModelUpdate, PullingState } from "@/hooks/use-update-checker";
import logoIcon from "@/assets/logo.png";
import ollamaDark from "@/assets/ollama.png";
import ollamaLight from "@/assets/ollama-light.png";
import "./update-notifications.css";

interface UpdateNotificationsProps {
  isOpen: boolean;
  onClose: () => void;
  appUpdate: AppUpdate | null;
  ollamaUpdates: OllamaModelUpdate[];
  pulling: PullingState | null;
  appDownloading: boolean;
  appPercent: number;
  onPullModel: (fullName: string) => void;
  onDownloadApp: (dmgUrl: string) => void;
  anchorLeft: number;
}

export function UpdateNotifications({
  isOpen, onClose,
  appUpdate, ollamaUpdates,
  pulling, appDownloading, appPercent,
  onPullModel, onDownloadApp,
  anchorLeft,
}: UpdateNotificationsProps) {
  const { t } = useTranslation();
  const [closing, setClosing] = useState(false);
  const listRef = useRef<HTMLDivElement>(null);

  const items = buildItems(appUpdate, ollamaUpdates);
  const maxDelay = items.length * 80;
  const closeDelay = maxDelay + 400;

  const handleClose = useCallback(() => {
    setClosing(true);
    setTimeout(() => {
      setClosing(false);
      onClose();
    }, closeDelay);
  }, [onClose, closeDelay]);

  useEffect(() => {
    if (!isOpen) return;
    const onEscape = (e: KeyboardEvent) => {
      if (e.code === "Escape") handleClose();
    };
    window.addEventListener("keydown", onEscape);
    return () => window.removeEventListener("keydown", onEscape);
  }, [isOpen, handleClose]);

  if (!isOpen) return null;

  return (
    <>
      <div className="update-overlay" onClick={handleClose} />
      <div ref={listRef} className="update-list" style={{ left: anchorLeft }}>
        {items.map((item, i) => (
          <BubbleItem
            key={item.id}
            item={item}
            index={i}
            closing={closing}
            totalCount={items.length}
            pulling={pulling}
            appDownloading={appDownloading}
            appPercent={appPercent}
            onPullModel={onPullModel}
            onDownloadApp={onDownloadApp}
            t={t}
          />
        ))}
      </div>
    </>
  );
}

interface ItemData {
  id: string;
  type: "app" | "ollama";
  name: string;
  sub: string;
  fullName?: string;
  dmgUrl?: string;
}

function buildItems(app: AppUpdate | null, models: OllamaModelUpdate[]): ItemData[] {
  const items: ItemData[] = [];
  if (app) {
    items.push({
      id: "app",
      type: "app",
      name: "CL-GO",
      sub: `Version ${app.version}`,
      dmgUrl: app.dmgUrl,
    });
  }
  for (const m of models) {
    items.push({
      id: m.fullName,
      type: "ollama",
      name: m.fullName,
      sub: m.family,
      fullName: m.fullName,
    });
  }
  return items;
}

interface BubbleItemProps {
  item: ItemData;
  index: number;
  closing: boolean;
  totalCount: number;
  pulling: PullingState | null;
  appDownloading: boolean;
  appPercent: number;
  onPullModel: (fullName: string) => void;
  onDownloadApp: (dmgUrl: string) => void;
  t: (kk: string, opts?: Record<string, string>) => string;
}

function BubbleItem({
  item, index, closing, totalCount,
  pulling, appDownloading, appPercent,
  onPullModel, onDownloadApp, t,
}: BubbleItemProps) {
  const delay = closing
    ? (totalCount - 1 - index) * 80
    : index * 80;

  const isOllamaPulling = pulling
    ? !pulling.fullName.localeCompare(item.fullName ?? "")
    : false;

  const showProgress = item.type === "app" ? appDownloading : isOllamaPulling;
  const percent = item.type === "app" ? appPercent : (pulling?.percent ?? 0);

  return (
    <div
      className={`update-bubble ${closing ? "bubble-closing" : ""}`}
      style={{ animationDelay: `${delay}ms` }}
    >
      {item.type === "app" ? (
        <img src={logoIcon} alt="" className="update-bubble-icon" />
      ) : (
        <ThemedIcon
          darkSrc={ollamaDark}
          lightSrc={ollamaLight}
          size={32}
          style={{ borderRadius: 8 }}
        />
      )}

      <div className="update-bubble-info">
        <span className="update-bubble-name">{item.name}</span>
        <span className="update-bubble-sub">{item.sub}</span>
      </div>

      {showProgress ? (
        <div className="update-bubble-progress">
          <div className="update-bubble-progress-bar">
            <div
              className="update-bubble-progress-fill"
              style={{ width: `${percent}%` }}
            />
          </div>
          <span className="update-bubble-progress-text">{percent}%</span>
        </div>
      ) : (
        <button
          className="update-bubble-btn"
          onClick={() => {
            if (item.type === "app" && item.dmgUrl) {
              onDownloadApp(item.dmgUrl);
            } else if (item.fullName) {
              onPullModel(item.fullName);
            }
          }}
        >
          {item.type === "app" ? t("updates.appUpdate") : t("updates.modelUpdate")}
        </button>
      )}
    </div>
  );
}

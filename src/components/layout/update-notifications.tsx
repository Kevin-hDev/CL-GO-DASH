import { useState, useEffect, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-shell";
import { ThemedIcon } from "@/components/ui/themed-icon";
import type { AppUpdate, OllamaModelUpdate, PullingState } from "@/hooks/use-update-checker";
import logoIcon from "@/assets/logo.png";
import ollamaDark from "@/assets/ollama.png";
import ollamaLight from "@/assets/ollama-light.png";
import "./update-notifications.css";

interface UpdateNotificationsProps {
  open: boolean;
  onClose: () => void;
  appUpdate: AppUpdate | null;
  ollamaUpdates: OllamaModelUpdate[];
  pulling: PullingState | null;
  onPullModel: (fullName: string) => void;
  anchorLeft: number;
}

export function UpdateNotifications({
  open: isOpen,
  onClose,
  appUpdate,
  ollamaUpdates,
  pulling,
  onPullModel,
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
      <div
        ref={listRef}
        className="update-list"
        style={{ left: anchorLeft }}
      >
        {items.map((item, i) => (
          <BubbleItem
            key={item.id}
            item={item}
            index={i}
            closing={closing}
            totalCount={items.length}
            pulling={pulling}
            onPullModel={onPullModel}
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
  downloadUrl?: string;
}

function buildItems(app: AppUpdate | null, models: OllamaModelUpdate[]): ItemData[] {
  const items: ItemData[] = [];
  if (app) {
    items.push({
      id: "app",
      type: "app",
      name: "CL-GO",
      sub: `Version ${app.version}`,
      downloadUrl: app.downloadUrl,
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
  onPullModel: (fullName: string) => void;
  t: (kk: string, opts?: Record<string, string>) => string;
}

function BubbleItem({ item, index, closing, totalCount, pulling, onPullModel, t }: BubbleItemProps) {
  const delay = closing
    ? (totalCount - 1 - index) * 80
    : index * 80;

  const isPulling = pulling
    ? !pulling.fullName.localeCompare(item.fullName ?? "")
    : false;

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

      {isPulling && pulling ? (
        <div className="update-bubble-progress">
          <div className="update-bubble-progress-bar">
            <div
              className="update-bubble-progress-fill"
              style={{ width: `${pulling.percent}%` }}
            />
          </div>
          <span className="update-bubble-progress-text">
            {pulling.percent}%
          </span>
        </div>
      ) : (
        <button
          className="update-bubble-btn"
          onClick={() => {
            if (item.type === "app" && item.downloadUrl) {
              open(item.downloadUrl);
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

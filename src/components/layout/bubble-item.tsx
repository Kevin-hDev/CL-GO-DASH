import { ThemedIcon } from "@/components/ui/themed-icon";
import type { PullingState } from "@/hooks/use-update-checker";
import logoIcon from "@/assets/logo.png";
import ollamaDark from "@/assets/ollama.png";
import ollamaLight from "@/assets/ollama-light.png";

export interface ItemData {
  id: string;
  type: "app" | "ollama-binary" | "ollama";
  name: string;
  sub: string;
  fullName?: string;
  assetUrl?: string;
}

interface BubbleItemProps {
  item: ItemData;
  index: number;
  closing: boolean;
  totalCount: number;
  pulling: PullingState | null;
  ollamaBinaryUpdating: boolean;
  ollamaBinaryPercent: number;
  appDownloading: boolean;
  appPercent: number;
  onPullModel: (fullName: string) => void;
  onDownloadApp: (dmgUrl: string) => void;
  onUpdateOllamaBinary: () => void;
  t: (kk: string, opts?: Record<string, string>) => string;
}

export function BubbleItem({
  item, index, closing, totalCount,
  pulling, ollamaBinaryUpdating, ollamaBinaryPercent,
  appDownloading, appPercent,
  onPullModel, onDownloadApp, onUpdateOllamaBinary, t,
}: BubbleItemProps) {
  const delay = closing
    ? (totalCount - 1 - index) * 80
    : index * 80;

  const isOllamaPulling = pulling
    ? !pulling.fullName.localeCompare(item.fullName ?? "")
    : false;

  const showProgress =
    item.type === "app" ? appDownloading
    : item.type === "ollama-binary" ? ollamaBinaryUpdating
    : isOllamaPulling;

  const percent =
    item.type === "app" ? appPercent
    : item.type === "ollama-binary" ? ollamaBinaryPercent
    : (pulling?.percent ?? 0);

  const buttonLabel =
    item.type === "app" ? t("updates.appUpdate")
    : item.type === "ollama-binary" ? t("updates.ollamaBinaryUpdate")
    : t("updates.modelUpdate");

  const handleClick = () => {
    if (item.type === "app" && item.assetUrl) {
      onDownloadApp(item.assetUrl);
    } else if (item.type === "ollama-binary") {
      onUpdateOllamaBinary();
    } else if (item.fullName) {
      onPullModel(item.fullName);
    }
  };

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
        <button className="update-bubble-btn" onClick={handleClick}>
          {buttonLabel}
        </button>
      )}
    </div>
  );
}

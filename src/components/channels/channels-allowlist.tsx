import { useState } from "react";
import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";

interface ChannelsAllowlistProps {
  allowlist: string[];
  onChange: (list: string[]) => void;
}

export function ChannelsAllowlist({ allowlist, onChange }: ChannelsAllowlistProps) {
  const { t } = useTranslation();
  const [input, setInput] = useState("");

  const handleAdd = () => {
    const id = input.trim();
    if (!id || allowlist.includes(id)) return;
    onChange([...allowlist, id]);
    setInput("");
  };

  const handleRemove = (id: string) => {
    onChange(allowlist.filter((a) => a !== id));
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      e.preventDefault();
      handleAdd();
    }
  };

  return (
    <div className="ch-allowlist">
      <div className="ch-allowlist-input-row">
        <input
          type="text"
          className="wk-input"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={t("channels.detail.allowlistPlaceholder")}
          style={{ flex: 1 }}
        />
        <button type="button" className="wk-btn-secondary" onClick={handleAdd} disabled={!input.trim()}>
          {t("channels.detail.allowlistAdd")}
        </button>
      </div>
      {allowlist.length > 0 && (
        <div className="ch-allowlist-tags">
          {allowlist.map((id) => (
            <span key={id} className="ch-allowlist-tag">
              {id}
              <button type="button" className="ch-allowlist-tag-remove" onClick={() => handleRemove(id)}>
                <X size={10} />
              </button>
            </span>
          ))}
        </div>
      )}
    </div>
  );
}

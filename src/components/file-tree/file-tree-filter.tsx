import { useRef } from "react";
import { useTranslation } from "react-i18next";
import { Search, X } from "lucide-react";

interface FileTreeFilterProps {
  value: string;
  onChange: (value: string) => void;
}

export function FileTreeFilter({ value, onChange }: FileTreeFilterProps) {
  const { t } = useTranslation();
  const inputRef = useRef<HTMLInputElement>(null);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      onChange("");
      inputRef.current?.blur();
    }
  };

  return (
    <div className="ft-filter-wrap">
      <Search size={13} style={{ position: "absolute", left: 10, color: "var(--ink-faint)" }} />
      <input
        ref={inputRef}
        className="ft-filter-input"
        style={{ paddingLeft: 30 }}
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={t("fileTree.filterPlaceholder")}
        spellCheck={false}
      />
      {value && (
        <button className="ft-filter-clear" onClick={() => onChange("")} type="button">
          <X size={13} />
        </button>
      )}
    </div>
  );
}

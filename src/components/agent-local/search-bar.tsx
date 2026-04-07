import { useRef, useEffect } from "react";
import { X } from "@/components/ui/icons";

interface SearchBarProps {
  query: string;
  onChange: (query: string) => void;
  matchCount: number;
  onClose: () => void;
}

export function SearchBar({ query, onChange, matchCount, onClose }: SearchBarProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => { inputRef.current?.focus(); }, []);

  return (
    <div style={{
      display: "flex", alignItems: "center", gap: 8,
      padding: "var(--space-sm) var(--space-md)",
      borderBottom: "1px solid var(--edge)", background: "var(--shell)",
    }}>
      <input
        ref={inputRef}
        value={query}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={(e) => { if (e.key.startsWith("Esc")) onClose(); }}
        placeholder="Rechercher..."
        style={{
          flex: 1, background: "transparent",
          fontSize: "var(--text-sm)", color: "var(--ink)",
          border: "none", outline: "none", fontFamily: "var(--font-sans)",
        }}
      />
      {query && (
        <span style={{ fontSize: "var(--text-xs)", color: "var(--ink-faint)" }}>
          {matchCount} résultat{matchCount !== 1 ? "s" : ""}
        </span>
      )}
      <button className="msg-action-btn" onClick={onClose}>
        <X size={14} />
      </button>
    </div>
  );
}

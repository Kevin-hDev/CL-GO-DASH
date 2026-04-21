import { useState, useEffect, useRef, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { useAgentSessions } from "@/hooks/use-agent-sessions";
import { useProjects } from "@/hooks/use-projects";
import "./search-dialog.css";

const MAX_RECENT = 6;

interface SearchDialogProps {
  open: boolean;
  onClose: () => void;
  onSelect: (sessionId: string) => void;
}

export function SearchDialog({ open, onClose, onSelect }: SearchDialogProps) {
  const { t } = useTranslation();
  const { sessions } = useAgentSessions();
  const { projects } = useProjects();
  const [query, setQuery] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);
  const [selectedIndex, setSelectedIndex] = useState(0);

  const projectMap = useMemo(() => {
    const map = new Map<string, string>();
    for (const p of projects) map.set(p.id, p.name);
    return map;
  }, [projects]);

  const sorted = useMemo(
    () => [...sessions].sort((a, b) => b.created_at.localeCompare(a.created_at)),
    [sessions],
  );

  const filtered = useMemo(() => {
    if (!query.trim()) return sorted.slice(0, MAX_RECENT);
    const q = query.toLowerCase();
    return sorted.filter((s) => {
      const name = s.name.toLowerCase();
      const proj = s.project_id ? (projectMap.get(s.project_id) ?? "").toLowerCase() : "";
      return name.includes(q) || proj.includes(q);
    });
  }, [sorted, query, projectMap]);

  useEffect(() => {
    if (open) {
      setQuery("");
      setSelectedIndex(0);
      requestAnimationFrame(() => inputRef.current?.focus());
    }
  }, [open]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      switch (e.key) {
        case "Escape":
          e.preventDefault();
          onClose();
          break;
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((i) => Math.min(i + 1, filtered.length - 1));
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((i) => Math.max(i - 1, 0));
          break;
        case "Enter":
          if (filtered.length > 0) {
            e.preventDefault();
            handleSelect(filtered[selectedIndex].id);
          }
          break;
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, filtered, selectedIndex, onClose]);

  const handleSelect = (id: string) => {
    onSelect(id);
    onClose();
  };

  if (!open) return null;

  return (
    <div className="search-overlay" onMouseDown={onClose}>
      <div className="search-dialog" onMouseDown={(e) => e.stopPropagation()}>
        <input
          ref={inputRef}
          className="search-input"
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder={t("search.placeholder")}
          spellCheck={false}
          autoComplete="off"
        />

        {filtered.length > 0 && (
          <div className="search-results">
            <div className="search-results-label">
              {query.trim() ? t("search.results") : t("search.recent")}
            </div>
            {filtered.map((session, i) => (
              <button
                key={session.id}
                className={`search-result-item${i - selectedIndex ? "" : " selected"}`}
                onMouseEnter={() => setSelectedIndex(i)}
                onClick={() => handleSelect(session.id)}
              >
                <SessionIcon />
                <span className="search-result-name">{session.name}</span>
                {session.project_id && projectMap.has(session.project_id) && (
                  <span className="search-result-project">
                    {projectMap.get(session.project_id)}
                  </span>
                )}
              </button>
            ))}
          </div>
        )}

        {filtered.length < 1 && query.trim() && (
          <div className="search-empty">{t("search.empty")}</div>
        )}
      </div>
    </div>
  );
}

function SessionIcon() {
  return (
    <svg width={16} height={16} viewBox="0 0 24 24" fill="none" stroke="currentColor"
      strokeWidth={1.5} strokeLinecap="round" strokeLinejoin="round" style={{ flexShrink: 0 }}>
      <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
    </svg>
  );
}

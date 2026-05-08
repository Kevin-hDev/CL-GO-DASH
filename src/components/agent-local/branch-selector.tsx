import { useState, useRef, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { GitBranch, Check, CaretDown, Plus } from "@/components/ui/icons";
import { useKeyboard } from "@/hooks/use-keyboard";
import { useClickOutside } from "@/hooks/use-click-outside";
import type { useGitBranch } from "@/hooks/use-git-branch";
import "./branch-selector.css";

type GitBranchHook = ReturnType<typeof useGitBranch>;

interface BranchSelectorProps {
  git: GitBranchHook;
  locked: boolean;
  onConflict: (branchName: string, dirtyCount: number) => void;
  onCreateRequest: () => void;
}

export function BranchSelector({
  git, locked, onConflict, onCreateRequest,
}: BranchSelectorProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const dropRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  useKeyboard({ onEscape: () => setOpen(false) });
  useClickOutside(dropRef, () => setOpen(false));

  useEffect(() => {
    if (open && searchRef.current) searchRef.current.focus();
  }, [open]);

  const handleSelect = useCallback(async (name: string) => {
    if (name === git.currentBranch) {
      setOpen(false);
      return;
    }
    const result = await git.checkout(name);
    if (result.ok) {
      setOpen(false);
      setSearch("");
    } else if (result.dirtyCount != null) {
      setOpen(false);
      onConflict(name, result.dirtyCount);
    }
  }, [git, onConflict]);

  if (!git.isGitRepo) return null;

  if (locked) {
    return (
      <div className="bs-row">
        <div className="bs-indicator">
          <GitBranch size={14} />
          <span>{git.currentBranch || t("branches.detachedHead")}</span>
        </div>
      </div>
    );
  }

  const lowerSearch = search.toLowerCase();
  const filteredBranches = git.branches.filter((b) =>
    b.name.toLowerCase().includes(lowerSearch),
  );
  const filteredWorktrees = git.worktrees.filter((w) =>
    w.branch.toLowerCase().includes(lowerSearch),
  );

  const label = git.currentBranch || t("branches.detachedHead");

  return (
    <div className="bs-row" ref={dropRef}>
      <button className="bs-btn" onClick={() => setOpen(!open)}>
        <GitBranch size={14} />
        <span>{label}</span>
        <CaretDown size={10} />
      </button>

      {open && (
        <div className="bs-dropdown">
          <input
            ref={searchRef}
            className="bs-dropdown-search"
            placeholder={t("branches.search")}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />

          <div className="bs-section-label">{t("branches.title")}</div>

          {filteredBranches.length === 0 && (
            <div className="bs-empty">{t("branches.noMatch")}</div>
          )}

          {filteredBranches.map((b) => (
            <div
              key={b.name}
              className={`bs-item ${b.is_current ? "bs-selected" : ""}`}
              role="button"
              tabIndex={0}
              onClick={() => void handleSelect(b.name)}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") void handleSelect(b.name);
              }}
            >
              <GitBranch size={14} />
              <div className="bs-item-info">
                <span className="bs-item-name">{b.name}</span>
                {b.is_current && b.dirty_count > 0 && (
                  <span className="bs-item-detail">
                    {t("branches.uncommitted", { count: b.dirty_count })}
                  </span>
                )}
              </div>
              {b.is_current && <Check size={14} />}
            </div>
          ))}

          {filteredWorktrees.length > 0 && (
            <>
              <div className="bs-sep" />
              <div className="bs-section-label">{t("branches.worktrees")}</div>
              {filteredWorktrees.map((w) => (
                <div key={w.path} className="bs-item" role="button" tabIndex={0}>
                  <GitBranch size={14} />
                  <div className="bs-item-info">
                    <span className="bs-item-name">{w.branch || w.path}</span>
                    <span className="bs-item-detail">{w.path}</span>
                  </div>
                </div>
              ))}
            </>
          )}

          <div className="bs-sep" />

          <div
            className="bs-item"
            role="button"
            tabIndex={0}
            onClick={() => { setOpen(false); onCreateRequest(); }}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") { setOpen(false); onCreateRequest(); }
            }}
          >
            <Plus size={14} />
            <span>{t("branches.createNew")}</span>
          </div>
        </div>
      )}
    </div>
  );
}

import { useState, useRef, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { GitBranch, CaretDown, Plus } from "@/components/ui/icons";
import { BranchSelectorBranchItem, BranchSelectorWorktreeItem } from "./branch-selector-items";
import { useKeyboard } from "@/hooks/use-keyboard";
import { useClickOutside } from "@/hooks/use-click-outside";
import type { useGitBranch } from "@/hooks/use-git-branch";
import "./branch-selector.css";

type GitBranchHook = ReturnType<typeof useGitBranch>;

interface BranchSelectorProps {
  git: GitBranchHook;
  locked: boolean;
  onConflict: (branchName: string, dirtyCount: number) => void;
  onWorktreeSelect: (path: string, branch: string) => void;
}

export function BranchSelector({
  git, locked, onConflict, onWorktreeSelect,
}: BranchSelectorProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [creating, setCreating] = useState(false);
  const [createName, setCreateName] = useState("");
  const [createError, setCreateError] = useState("");
  const dropRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const createRef = useRef<HTMLInputElement>(null);

  useKeyboard({ onEscape: () => {
    if (creating) {
      setCreating(false);
      setCreateName("");
      setCreateError("");
    } else {
      setOpen(false);
    }
  }});
  useClickOutside(dropRef, () => { setOpen(false); setCreating(false); });

  useEffect(() => {
    if (open && !creating && searchRef.current) searchRef.current.focus();
  }, [open, creating]);

  useEffect(() => {
    if (creating && createRef.current) createRef.current.focus();
  }, [creating]);

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

  const handleCreate = useCallback(async () => {
    const name = createName.trim();
    if (!name) return;
    const ok = await git.create(name);
    if (ok) {
      setOpen(false);
      setCreating(false);
      setCreateName("");
      setCreateError("");
      setSearch("");
    } else {
      setCreateError(t("branches.createError"));
    }
  }, [createName, git, t]);

  const handleWorktreeSelect = useCallback((path: string, branch: string) => {
    setOpen(false);
    onWorktreeSelect(path, branch);
  }, [onWorktreeSelect]);

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
    !w.is_current && `${w.branch} ${w.path}`.toLowerCase().includes(lowerSearch),
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
            <BranchSelectorBranchItem
              key={b.name}
              branch={b}
              dirtyLabel={b.is_current && b.dirty_count > 0
                ? t("branches.uncommitted", { count: b.dirty_count })
                : undefined}
              onSelect={(name) => void handleSelect(name)}
            />
          ))}

          {filteredWorktrees.length > 0 && (
            <>
              <div className="bs-sep" />
              <div className="bs-section-label">{t("branches.worktrees")}</div>
              {filteredWorktrees.map((w) => (
                <BranchSelectorWorktreeItem
                  key={w.path}
                  worktree={w}
                  onSelect={handleWorktreeSelect}
                />
              ))}
            </>
          )}

          <div className="bs-sep" />

          {creating ? (
            <div className="bs-create-form">
              <input
                ref={createRef}
                className="bs-create-input"
                placeholder={t("branches.createPlaceholder")}
                value={createName}
                onChange={(e) => { setCreateName(e.target.value); setCreateError(""); }}
                onKeyDown={(e) => {
                  if (e.key === "Enter") void handleCreate();
                  if (e.key === "Escape") {
                    e.stopPropagation();
                    setCreating(false);
                    setCreateName("");
                    setCreateError("");
                  }
                }}
              />
              {createError && <div className="bs-create-error">{createError}</div>}
            </div>
          ) : (
            <div
              className="bs-item"
              role="button"
              tabIndex={0}
              onClick={() => setCreating(true)}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") setCreating(true);
              }}
            >
              <Plus size={14} />
              <span>{t("branches.createNew")}</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

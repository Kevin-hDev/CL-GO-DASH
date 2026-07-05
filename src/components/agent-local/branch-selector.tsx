import { useState, useRef, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { GitBranch, CaretDown } from "@/components/ui/icons";
import { BranchSelectorCreateForm } from "./branch-selector-create-form";
import { BranchSelectorBranchItem, BranchSelectorCreateItem, BranchSelectorLockedIndicator, BranchSelectorWorktreeItem } from "./branch-selector-items";
import { branchCreateErrorKey, getVisibleBranchOptions } from "./branch-selector-utils";
import { useKeyboard } from "@/hooks/use-keyboard";
import { useClickOutside } from "@/hooks/use-click-outside";
import type { useGitBranch } from "@/hooks/use-git-branch";
import { showToast } from "@/lib/toast-emitter";
import { validateBranchName } from "@/lib/branch-name";
import "./branch-selector.css";
type GitBranchHook = ReturnType<typeof useGitBranch>;

interface BranchSelectorProps {
  git: GitBranchHook;
  locked: boolean;
  onConflict: (branchName: string, dirtyCount: number) => void;
  onWorktreeSelect: (path: string, branch: string) => void;
  onGithubAuthRequired: () => void;
}

export function BranchSelector({
  git, locked, onConflict, onWorktreeSelect, onGithubAuthRequired,
}: BranchSelectorProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [creating, setCreating] = useState(false);
  const [createName, setCreateName] = useState("");
  const [createError, setCreateError] = useState("");
  const [isCreating, setIsCreating] = useState(false);
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
    if (isCreating) return;
    const name = createName.trim();
    const validation = validateBranchName(name);
    if (!validation.valid) {
      setCreateError(t(branchCreateErrorKey(validation.reason)));
      return;
    }

    setIsCreating(true);
    try {
      const result = await git.create(name);
      if (result.ok) {
        showToast(`${t("branches.bubbleCreated")}: ${name}`, "success", 3000);
        setOpen(false);
        setCreating(false);
        setCreateName("");
        setCreateError("");
        setSearch("");
      } else if (result.reason === "github_auth_required") {
        setOpen(false);
        setCreating(false);
        setCreateName("");
        setCreateError("");
        onGithubAuthRequired();
      } else {
        setCreateError(t(branchCreateErrorKey(result.kind)));
      }
    } finally {
      setIsCreating(false);
    }
  }, [createName, git, isCreating, onGithubAuthRequired, t]);

  const handleWorktreeSelect = useCallback((path: string, branch: string) => {
    setOpen(false);
    onWorktreeSelect(path, branch);
  }, [onWorktreeSelect]);
  const cancelCreate = useCallback(() => {
    if (isCreating) return;
    setCreating(false);
    setCreateName("");
    setCreateError("");
  }, [isCreating]);

  if (!git.isGitRepo) return null;

  if (locked) {
    return <BranchSelectorLockedIndicator label={git.currentBranch || t("branches.detachedHead")} />;
  }

  const { filteredBranches, filteredWorktrees } = getVisibleBranchOptions(
    git.branches,
    git.worktrees,
    search,
  );

  const label = git.currentBranch || t("branches.detachedHead");
  return (
    <div className="bs-row" ref={dropRef}>
      <button className="bs-btn" onClick={() => setOpen(!open)}>
        <GitBranch size="var(--icon-sm)" />
        <span>{label}</span>
        <CaretDown size="var(--icon-2xs)" />
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
            <BranchSelectorCreateForm
              inputRef={createRef}
              value={createName}
              error={createError}
              isCreating={isCreating}
              placeholder={t("branches.createPlaceholder")}
              onValueChange={(value) => { setCreateName(value); setCreateError(""); }}
              onSubmit={() => void handleCreate()}
              onCancel={cancelCreate}
            />
          ) : (
            <BranchSelectorCreateItem
              label={t("branches.createNew")}
              onStart={() => setCreating(true)}
            />
          )}
        </div>
      )}
    </div>
  );
}

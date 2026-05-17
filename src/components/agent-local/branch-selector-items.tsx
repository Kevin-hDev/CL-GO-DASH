import { GitBranch, Check } from "@/components/ui/icons";

interface BranchItem {
  name: string;
  is_current: boolean;
  dirty_count: number;
}

interface WorktreeItem {
  path: string;
  branch: string;
}

interface BranchSelectorBranchItemProps {
  branch: BranchItem;
  dirtyLabel?: string;
  onSelect: (name: string) => void;
}

interface BranchSelectorWorktreeItemProps {
  worktree: WorktreeItem;
  onSelect: (path: string, branch: string) => void;
}

export function BranchSelectorBranchItem({
  branch,
  dirtyLabel,
  onSelect,
}: BranchSelectorBranchItemProps) {
  return (
    <div
      className={`bs-item ${branch.is_current ? "bs-selected" : ""}`}
      role="button"
      tabIndex={0}
      onClick={() => onSelect(branch.name)}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") onSelect(branch.name);
      }}
    >
      <GitBranch size={14} />
      <div className="bs-item-info">
        <span className="bs-item-name">{branch.name}</span>
        {dirtyLabel && <span className="bs-item-detail">{dirtyLabel}</span>}
      </div>
      {branch.is_current && <Check size={14} />}
    </div>
  );
}

export function BranchSelectorWorktreeItem({
  worktree,
  onSelect,
}: BranchSelectorWorktreeItemProps) {
  const name = worktree.branch || worktree.path;

  return (
    <div
      className="bs-item"
      role="button"
      tabIndex={0}
      onClick={() => onSelect(worktree.path, worktree.branch)}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") onSelect(worktree.path, worktree.branch);
      }}
    >
      <GitBranch size={14} />
      <div className="bs-item-info">
        <span className="bs-item-name">{name}</span>
        <span className="bs-item-detail">{worktree.path}</span>
      </div>
    </div>
  );
}

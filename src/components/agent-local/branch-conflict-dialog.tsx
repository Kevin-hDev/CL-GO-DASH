import { useEffect, useRef, useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { X } from "@/components/ui/icons";
import { useKeyboard } from "@/hooks/use-keyboard";
import "./branch-conflict-dialog.css";

interface DirtyFile {
  path: string;
  status: string;
  additions: number;
  deletions: number;
}

interface BranchConflictDialogProps {
  targetBranch: string;
  dirtyCount: number;
  projectPath: string;
  onCancel: () => void;
  onCommitAndSwitch: (branch: string) => void;
}

export function BranchConflictDialog({
  targetBranch, dirtyCount, projectPath, onCancel, onCommitAndSwitch,
}: BranchConflictDialogProps) {
  const { t } = useTranslation();
  const overlayRef = useRef<HTMLDivElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const [files, setFiles] = useState<DirtyFile[]>([]);

  useKeyboard({ onEscape: onCancel });

  useEffect(() => {
    void invoke<DirtyFile[]>("list_git_dirty_files", { path: projectPath })
      .then(setFiles)
      .catch((e) => console.error("list_git_dirty_files:", e));
  }, [projectPath]);

  useEffect(() => {
    dialogRef.current?.focus();
  }, []);

  const handleOverlayClick = useCallback((e: React.MouseEvent) => {
    if (e.target === overlayRef.current) onCancel();
  }, [onCancel]);

  const statLabel = (f: DirtyFile) => ({
    add: `+${f.additions}`,
    del: `-${f.deletions}`,
  });

  return (
    <div className="bcd-overlay" ref={overlayRef} role="presentation" onClick={handleOverlayClick} onKeyDown={() => {}}>
      <div className="bcd-dialog" ref={dialogRef} tabIndex={-1}>
        <button className="bcd-close" onClick={onCancel} type="button">
          <X size={16} />
        </button>

        <div className="bcd-title">{t("branches.conflictTitle")}</div>

        <div className="bcd-description">{t("branches.conflictDescription")}</div>

        <div className="bcd-file-list">
          {files.map((f) => {
            const stat = statLabel(f);
            return (
              <div key={f.path} className="bcd-file">
                <span>{f.path}</span>
                <span className="bcd-file-stat">
                  <span className="bcd-file-stat-add">{stat.add}</span>{" "}
                  <span className="bcd-file-stat-del">{stat.del}</span>
                </span>
              </div>
            );
          })}
          {files.length === 0 && (
            <div className="bcd-hint">
              {t("branches.uncommitted", { count: dirtyCount })}
            </div>
          )}
        </div>

        <div className="bcd-hint">{t("branches.commitRequired")}</div>

        <div className="bcd-actions">
          <button className="bcd-btn" onClick={onCancel} type="button">
            {t("branches.conflictCancel")}
          </button>
          <button
            className="bcd-btn bcd-btn-primary"
            onClick={() => onCommitAndSwitch(targetBranch)}
            type="button"
          >
            {t("branches.conflictCommitAndSwitch")}
          </button>
        </div>
      </div>
    </div>
  );
}

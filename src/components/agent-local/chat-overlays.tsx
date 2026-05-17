import { FilePreview } from "./file-preview";
import { SwitchModelDialog } from "./switch-model-dialog";
import { WorktreeSwitchDialog } from "./worktree-switch-dialog";
import type { DroppedFile } from "@/hooks/use-file-drop";
import type { WorktreeSwitchTarget } from "@/hooks/use-worktree-session-switch";

interface ChatOverlaysProps {
  preview: DroppedFile | null;
  currentModel: string;
  pendingSwitch: { model: string; provider: string } | null;
  pendingWorktreeSwitch: WorktreeSwitchTarget | null;
  onClosePreview: () => void;
  onCancelSwitch: () => void;
  onCancelWorktreeSwitch: () => void;
  onNewSession: (remember: boolean) => void;
  onContinue: (remember: boolean) => void;
  onNewWorktreeSession: () => void;
}

export function ChatOverlays({
  preview,
  currentModel,
  pendingSwitch,
  pendingWorktreeSwitch,
  onClosePreview,
  onCancelSwitch,
  onCancelWorktreeSwitch,
  onNewSession,
  onContinue,
  onNewWorktreeSession,
}: ChatOverlaysProps) {
  return (
    <>
      {preview && (
        <FilePreview
          name={preview.name}
          path={preview.path}
          thumbnail={preview.preview}
          isImage={!!preview.preview}
          onClose={onClosePreview}
        />
      )}
      {pendingSwitch && (
        <SwitchModelDialog
          fromModel={currentModel}
          toModel={pendingSwitch.model}
          onNewSession={onNewSession}
          onContinue={onContinue}
          onCancel={onCancelSwitch}
        />
      )}
      {pendingWorktreeSwitch && (
        <WorktreeSwitchDialog
          branch={pendingWorktreeSwitch.branch}
          path={pendingWorktreeSwitch.path}
          onCancel={onCancelWorktreeSwitch}
          onNewSession={onNewWorktreeSession}
        />
      )}
    </>
  );
}

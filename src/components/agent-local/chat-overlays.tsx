import { FilePreview } from "./file-preview";
import { SwitchModelDialog } from "./switch-model-dialog";
import type { DroppedFile } from "@/hooks/use-file-drop";

interface ChatOverlaysProps {
  preview: DroppedFile | null;
  currentModel: string;
  pendingSwitch: { model: string; provider: string } | null;
  onClosePreview: () => void;
  onCancelSwitch: () => void;
  onNewSession: (remember: boolean) => void;
  onContinue: (remember: boolean) => void;
}

export function ChatOverlays({
  preview,
  currentModel,
  pendingSwitch,
  onClosePreview,
  onCancelSwitch,
  onNewSession,
  onContinue,
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
    </>
  );
}

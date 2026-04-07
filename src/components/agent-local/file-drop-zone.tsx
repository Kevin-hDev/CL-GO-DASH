import { useCallback, type ReactNode } from "react";

interface FileDropZoneProps {
  dragging: boolean;
  onDragChange: (dragging: boolean) => void;
  onDrop: (files: FileList) => void;
  children: ReactNode;
}

export function FileDropZone({ dragging, onDragChange, onDrop, children }: FileDropZoneProps) {
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    onDragChange(true);
  }, [onDragChange]);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    if (e.currentTarget.contains(e.relatedTarget as Node)) return;
    onDragChange(false);
  }, [onDragChange]);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    onDragChange(false);
    if (e.dataTransfer.files.length > 0) onDrop(e.dataTransfer.files);
  }, [onDragChange, onDrop]);

  return (
    <div
      style={{ position: "relative", height: "100%" }}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      {children}
      {dragging && (
        <div style={{
          position: "absolute", inset: 0, zIndex: 40,
          display: "flex", alignItems: "center", justifyContent: "center",
          background: "rgba(28, 28, 34, 0.8)",
          border: "2px dashed var(--pulse)",
          borderRadius: "var(--radius-md)",
        }}>
          <span style={{ fontSize: "var(--text-sm)", color: "var(--pulse)" }}>
            Déposer les fichiers ici
          </span>
        </div>
      )}
    </div>
  );
}

import { useEffect, type ReactNode } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";

interface FileDropZoneProps {
  dragging: boolean;
  onDragChange: (dragging: boolean) => void;
  onDropPaths: (paths: string[]) => void;
  children: ReactNode;
}

export function FileDropZone({ dragging, onDragChange, onDropPaths, children }: FileDropZoneProps) {
  useEffect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "over") {
        onDragChange(true);
      } else if (event.payload.type === "drop") {
        onDragChange(false);
        if (event.payload.paths.length > 0) {
          onDropPaths(event.payload.paths);
        }
      } else {
        onDragChange(false);
      }
    });

    return () => { unlisten.then((fn) => fn()).catch(() => {}); };
  }, [onDragChange, onDropPaths]);

  return (
    <div style={{ position: "relative", height: "100%" }}>
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

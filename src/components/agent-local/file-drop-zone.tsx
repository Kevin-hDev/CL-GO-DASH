import { useEffect, useRef, type ReactNode } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";

interface FileDropZoneProps {
  dragging: boolean;
  onDragChange: (dragging: boolean) => void;
  onDropPaths: (paths: string[]) => void;
  children: ReactNode;
}

export function FileDropZone({ dragging, onDragChange, onDropPaths, children }: FileDropZoneProps) {
  const dragRef = useRef(onDragChange);
  const dropRef = useRef(onDropPaths);
  dragRef.current = onDragChange;
  dropRef.current = onDropPaths;

  useEffect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "over") {
        dragRef.current(true);
      } else if (event.payload.type === "drop") {
        dragRef.current(false);
        if (event.payload.paths.length > 0) {
          dropRef.current(event.payload.paths);
        }
      } else {
        dragRef.current(false);
      }
    });

    return () => { unlisten.then((fn) => fn()).catch(() => {}); };
  }, []);

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

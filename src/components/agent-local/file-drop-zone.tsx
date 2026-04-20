import { useEffect, useRef, type ReactNode } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { isInternalDrag } from "@/lib/internal-drag";

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
      if (isInternalDrag()) return;
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
    <div style={{ position: "relative", height: "100%", overflow: "hidden", borderRadius: "inherit" }}>
      {children}
      {dragging && (
        <div style={{
          position: "absolute", top: 2, right: 2, bottom: 2, left: 12, zIndex: 40,
          pointerEvents: "none",
          border: "2px solid var(--pulse)",
          borderRadius: "14px",
        }} />
      )}
    </div>
  );
}

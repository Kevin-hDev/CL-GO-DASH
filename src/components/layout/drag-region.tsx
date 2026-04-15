import { getCurrentWindow } from "@tauri-apps/api/window";

interface DragRegionProps {
  height?: number;
  style?: React.CSSProperties;
}

export function DragRegion({ height = 32, style }: DragRegionProps) {
  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.button !== 0) return;
    getCurrentWindow().startDragging().catch(() => { /* ignore */ });
  };

  const handleDoubleClick = () => {
    const win = getCurrentWindow();
    win.isMaximized()
      .then((m) => (m ? win.unmaximize() : win.maximize()))
      .catch(() => { /* ignore */ });
  };

  return (
    <div
      onMouseDown={handleMouseDown}
      onDoubleClick={handleDoubleClick}
      style={{
        height,
        flexShrink: 0,
        userSelect: "none",
        WebkitUserSelect: "none",
        cursor: "default",
        ...style,
      }}
    />
  );
}

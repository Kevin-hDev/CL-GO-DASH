import { useEffect, useState } from "react";
import type { FileOperation, FilePreviewActiveTab } from "@/types/file-preview";
import type { PreviewEditor } from "@/types/file-preview";
import { FileTabMenu } from "./file-tab-menu";
import { FilePreviewTab } from "./file-preview-tab";

interface FilePreviewTabsProps {
  tabs: FileOperation[];
  activeTab: FilePreviewActiveTab;
  editors: PreviewEditor[];
  onSelect: (id: FilePreviewActiveTab) => void;
  onClose: (id: string) => void;
  onOpenDefault: (operation: FileOperation) => void;
  onOpenWith: (operation: FileOperation, editorId: string) => void;
}

export function FilePreviewTabs({
  tabs,
  activeTab,
  editors,
  onSelect,
  onClose,
  onOpenDefault,
  onOpenWith,
}: FilePreviewTabsProps) {
  const [menu, setMenu] = useState<{ x: number; y: number; operation: FileOperation } | null>(null);

  useEffect(() => {
    if (!menu) return;
    const close = () => setMenu(null);
    window.addEventListener("pointerdown", close);
    return () => window.removeEventListener("pointerdown", close);
  }, [menu]);

  return (
    <div className="fp-tabs">
      <FilePreviewTab
        summary
        label="R"
        active={activeTab === "summary"}
        onSelect={() => onSelect("summary")}
      />
      <div className="fp-tabs-files">
        {tabs.map((tab) => (
          <FilePreviewTab
            key={tab.id}
            operation={tab}
            label={tab.name}
            active={activeTab === tab.id}
            onSelect={() => onSelect(tab.id)}
            onClose={() => onClose(tab.id)}
            onContextMenu={(event) => {
              event.preventDefault();
              event.stopPropagation();
              setMenu({ x: event.clientX, y: event.clientY, operation: tab });
            }}
          />
        ))}
      </div>
      {menu && (
        <FileTabMenu
          x={menu.x}
          y={menu.y}
          editors={editors}
          onOpen={() => {
            onOpenDefault(menu.operation);
            setMenu(null);
          }}
          onOpenWith={(editorId) => {
            onOpenWith(menu.operation, editorId);
            setMenu(null);
          }}
          onClose={() => setMenu(null)}
        />
      )}
    </div>
  );
}

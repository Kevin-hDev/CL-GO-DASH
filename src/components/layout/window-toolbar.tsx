import { SidebarToggleIcon, ArrowLeftIcon, ArrowRightIcon } from "./toolbar-icons";
import { ComposeIcon } from "@/components/ui/compose-icon";
import "./window-toolbar.css";

interface WindowToolbarProps {
  sidebarOpen: boolean;
  onToggleSidebar: () => void;
  onBack: () => void;
  onForward: () => void;
  onNewSession: () => void;
  canGoBack: boolean;
  canGoForward: boolean;
}

export function WindowToolbar({
  sidebarOpen, onToggleSidebar,
  onBack, onForward, onNewSession,
  canGoBack, canGoForward,
}: WindowToolbarProps) {
  return (
    <div className="window-toolbar">
      <button className="toolbar-btn" onClick={onToggleSidebar} title="Toggle sidebar">
        <SidebarToggleIcon size={16} />
      </button>
      <button className="toolbar-btn" onClick={onBack} disabled={!canGoBack} title="Retour">
        <ArrowLeftIcon size={16} />
      </button>
      <button className="toolbar-btn" onClick={onForward} disabled={!canGoForward} title="Suivant">
        <ArrowRightIcon size={16} />
      </button>
      {!sidebarOpen && (
        <button className="toolbar-btn" onClick={onNewSession} title="Nouvelle session">
          <ComposeIcon size={16} />
        </button>
      )}
    </div>
  );
}

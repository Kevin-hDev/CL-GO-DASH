import { useTranslation } from "react-i18next";
import { TerminalSquare } from "lucide-react";
import { Plus } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { MOD } from "@/lib/platform";
import "./tab-bar-actions.css";

interface TabBarActionsProps {
  canAddTab: boolean;
  sessionId: string | null;
  terminalOpen: boolean;
  onAdd: () => void;
  onToggleTerminal: () => void;
}

export function TabBarActions({
  canAddTab,
  sessionId,
  terminalOpen,
  onAdd,
  onToggleTerminal,
}: TabBarActionsProps) {
  const { t } = useTranslation();

  return (
    <>
      {canAddTab && (
        <button className="tab-add" onClick={onAdd}>
          <Plus size={14} />
        </button>
      )}
      {sessionId && (
        <span className="tab-actions">
          <Tooltip label={`${t("settings.shortcuts.toggleTerminal")} (${MOD}J)`} align="right">
            <button
              className={`tab-action-btn ${terminalOpen ? "active" : ""}`}
              onClick={(event) => {
                event.stopPropagation();
                onToggleTerminal();
              }}
            >
              <TerminalSquare size={18} />
            </button>
          </Tooltip>
        </span>
      )}
    </>
  );
}

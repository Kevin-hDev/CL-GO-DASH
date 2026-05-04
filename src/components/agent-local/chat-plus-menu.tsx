import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus, Image, Plugs, PuzzlePiece, CaretRight } from "@/components/ui/icons";
import { useConnectors } from "@/hooks/use-connectors";
import { McpIcon } from "@/lib/mcp-icons";
import "./chat-plus-menu.css";

interface ChatPlusMenuProps {
  onFileImport: () => void;
}

export function ChatPlusMenu({ onFileImport }: ChatPlusMenuProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [submenu, setSubmenu] = useState<"connectors" | "plugins" | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const { configured, toggleChatEnabled } = useConnectors();

  const close = useCallback(() => { setOpen(false); setSubmenu(null); }, []);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") close(); };
    const onClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) close();
    };
    window.addEventListener("keydown", onKey);
    document.addEventListener("mousedown", onClick);
    return () => { window.removeEventListener("keydown", onKey); document.removeEventListener("mousedown", onClick); };
  }, [open, close]);

  const handleFileImport = () => { close(); onFileImport(); };

  const connectedItems = configured.filter((c) => c.status === "connected");

  const submenuLeft = dropdownRef.current
    ? dropdownRef.current.offsetWidth + 4
    : 244;

  return (
    <div className="cpm-wrapper" ref={menuRef}>
      <button className="chat-plus-btn" onClick={() => setOpen(!open)} type="button">
        <Plus size={16} />
      </button>

      {open && (
        <div className="cpm-dropdown" ref={dropdownRef}>
          <button type="button" className="cpm-item" onClick={handleFileImport}>
            <Image size={16} weight="regular" />
            <span>{t("chatMenu.addFile")}</span>
          </button>

          <div className="cpm-separator" />

          <button
            type="button"
            className={`cpm-item cpm-has-sub ${submenu === "connectors" ? "active" : ""}`}
            onMouseEnter={() => setSubmenu("connectors")}
          >
            <Plugs size={16} weight="regular" />
            <span>{t("chatMenu.connectors")}</span>
            <CaretRight size={12} className="cpm-caret" />
          </button>

          <button
            type="button"
            className={`cpm-item cpm-has-sub ${submenu === "plugins" ? "active" : ""}`}
            onMouseEnter={() => setSubmenu("plugins")}
          >
            <PuzzlePiece size={16} weight="regular" />
            <span>{t("chatMenu.plugins")}</span>
            <CaretRight size={12} className="cpm-caret" />
          </button>
        </div>
      )}

      {open && submenu === "connectors" && (
        <div className="cpm-submenu" style={{ left: submenuLeft }} onMouseLeave={() => setSubmenu(null)}>
          {connectedItems.length === 0 ? (
            <div className="cpm-sub-empty">{t("chatMenu.noConnectors")}</div>
          ) : (
            connectedItems.map((c) => (
              <button key={c.id} type="button" className="cpm-sub-item" onClick={() => toggleChatEnabled(c.id)}>
                <McpIcon connectorId={c.id} displayName={c.display_name} size={18} />
                <span className={c.enabled_in_chat ? "" : "cpm-disabled"}>{c.display_name}</span>
                <div className={`cpm-toggle ${c.enabled_in_chat ? "on" : ""}`}>
                  <div className="cpm-toggle-dot" />
                </div>
              </button>
            ))
          )}
        </div>
      )}

      {open && submenu === "plugins" && (
        <div className="cpm-submenu" style={{ left: submenuLeft }} onMouseLeave={() => setSubmenu(null)}>
          <div className="cpm-sub-empty">{t("chatMenu.pluginsEmpty")}</div>
        </div>
      )}
    </div>
  );
}

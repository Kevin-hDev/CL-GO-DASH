import { useTranslation } from "react-i18next";
import type { RefObject, MouseEvent } from "react";
import { ChatsCircle, DotsThreeVertical } from "@/components/ui/icons";
import { ChannelIcon } from "@/components/channels/channel-icon";
import type { AgentSessionMeta } from "@/types/agent";
import type { ChannelType } from "@/types/channels";
import { displaySessionName } from "@/lib/utils";
import "./conversation-session-item.css";

interface ConversationSessionItemProps {
  session: AgentSessionMeta;
  active: boolean;
  renaming: boolean;
  inputRef: RefObject<HTMLInputElement | null>;
  onSelect: (id: string) => void;
  onRenameSubmit: (id: string, value: string) => void;
  onCancelRename: () => void;
  onMenu: (e: MouseEvent, id: string) => void;
}

export function ConversationSessionItem({
  session, active, renaming, inputRef,
  onSelect, onRenameSubmit, onCancelRename, onMenu,
}: ConversationSessionItemProps) {
  const { t } = useTranslation();
  const channelId = gatewayChannelId(session.gateway_channel_key);

  return (
    <div
      className={`conv-item conv-session-indented ${active ? "active" : ""}`}
      role="button"
      tabIndex={0}
      onClick={() => onSelect(session.id)}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") onSelect(session.id);
      }}
    >
      {renaming ? (
        <input
          ref={inputRef}
          className="conv-rename"
          defaultValue={session.name}
          onFocus={(e) => e.target.select()}
          onBlur={(e) => onRenameSubmit(session.id, e.target.value)}
          onKeyDown={(e) => {
            if (e.key.startsWith("Ent")) onRenameSubmit(session.id, e.currentTarget.value);
            if (e.key.startsWith("Esc")) onCancelRename();
          }}
        />
      ) : (
        <>
          <ChatsCircle size={14} weight={active ? "fill" : "regular"} className="conv-icon" />
          <span className="conv-name">{displaySessionName(session.name, t)}</span>
          {session.is_gateway && channelId && (
            <ChannelIcon channelId={channelId} size={18} className="conv-gateway-icon" />
          )}
          <button className="conv-session-menu-btn" onClick={(e) => onMenu(e, session.id)}>
            <DotsThreeVertical size={14} />
          </button>
        </>
      )}
    </div>
  );
}

function gatewayChannelId(key?: string): ChannelType | null {
  const id = key?.split("/")[0];
  return id === "telegram" || id === "discord" || id === "slack" ? id : null;
}

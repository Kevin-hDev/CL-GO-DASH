import type { SessionEntry, ToolCall } from "@/types/session";
import { CommandBubble } from "./command-bubble";

interface ConversationViewProps {
  entries: SessionEntry[];
}

type GroupedItem =
  | { type: "message"; role: string; content: string }
  | { type: "tools"; tools: ToolCall[] };

function groupEntries(entries: SessionEntry[]): GroupedItem[] {
  const groups: GroupedItem[] = [];
  let toolBatch: ToolCall[] = [];

  function flushTools() {
    if (toolBatch.length > 0) {
      groups.push({ type: "tools", tools: [...toolBatch] });
      toolBatch = [];
    }
  }

  for (const entry of entries) {
    if (entry.kind === "tool") {
      toolBatch.push(entry);
    } else {
      flushTools();
      groups.push({ type: "message", role: entry.role, content: entry.content });
    }
  }
  flushTools();
  return groups;
}

export function ConversationView({ entries }: ConversationViewProps) {
  const groups = groupEntries(entries);

  return (
    <>
      {groups.map((group, i) => {
        if (group.type === "tools") {
          return <CommandBubble key={`tools-${i}`} tools={group.tools} />;
        }
        return (
          <div key={`msg-${i}`} className={`sd-msg sd-msg-${group.role}`}>
            <div className="sd-msg-role">
              {group.role === "user" ? "Prompt" : "Jackson"}
            </div>
            <div className="sd-msg-text">{group.content}</div>
          </div>
        );
      })}
    </>
  );
}

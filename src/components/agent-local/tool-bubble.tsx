import { useMemo } from "react";
import type { ToolActivity } from "@/hooks/agent-chat-utils";
import type { ToolActivityRecord } from "@/types/agent";
import { groupToolActivities } from "@/lib/tool-activity-summary";
import { isHiddenAgentTool } from "@/lib/hidden-agent-tools";
import { ToolActivityGroupList } from "./tool-activity-group";
import {
  savedToolToRenderable,
  streamToolToRenderable,
} from "./tool-detail-row";
import "./tool-bubble.css";
import "./tool-bubble-detail.css";

export function ToolBubble({
  tools,
  onFilePreview,
  projectPath,
}: {
  tools: ToolActivity[];
  onFilePreview?: (path: string) => void;
  projectPath?: string;
}) {
  const visibleTools = useMemo(() => tools.filter(isVisibleTool), [tools]);
  const groups = useMemo(
    () => groupToolActivities(visibleTools.map(streamToolToRenderable)),
    [visibleTools],
  );
  if (groups.length === 0) return null;
  return (
    <div className="chat-bubble">
      <ToolActivityGroupList groups={groups} onFilePreview={onFilePreview} projectPath={projectPath} />
    </div>
  );
}

export function SavedToolBubble({
  tools,
  onFilePreview,
  projectPath,
}: {
  tools: ToolActivityRecord[];
  onFilePreview?: (path: string) => void;
  projectPath?: string;
}) {
  const visibleTools = useMemo(() => tools.filter(isVisibleTool), [tools]);
  const groups = useMemo(
    () => groupToolActivities(visibleTools.map(savedToolToRenderable)),
    [visibleTools],
  );
  if (groups.length === 0) return null;
  return (
    <div className="chat-bubble">
      <ToolActivityGroupList groups={groups} onFilePreview={onFilePreview} projectPath={projectPath} />
    </div>
  );
}

function isVisibleTool(tool: { name: string }) {
  return !isHiddenAgentTool(tool.name);
}

import type { SubagentInfo } from "@/types/agent";

type SubagentType = SubagentInfo["type"];

const LEGACY_NAMES = new Set(["", "agent", "explore", "explorer", "coder"]);

export function subagentDefaultName(type: SubagentType): string {
  return type === "coder" ? "Claudiator" : "Geminitor";
}

export function subagentDisplayName(agent: Pick<SubagentInfo, "name" | "type">): string {
  const name = agent.name.trim();
  return LEGACY_NAMES.has(name.toLowerCase()) ? subagentDefaultName(agent.type) : name;
}

export function subagentColorKey(
  agent: Pick<SubagentInfo, "type" | "colorKey">,
): string {
  return agent.colorKey || (agent.type === "coder" ? "claudiator" : "geminitor");
}

export function subagentSecondaryText(agent: SubagentInfo): string {
  return agent.description || agent.lastActivity?.label || agent.summary || agent.promptPreview || "...";
}

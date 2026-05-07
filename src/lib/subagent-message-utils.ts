import type { AgentMessage, SubagentInfo } from "@/types/agent";

const SUBAGENT_REPORT_PREFIX = "[Rapport du sous-agent";
const SUBAGENT_SYNTHESIS_PREFIX = "Tous les sous-agents ont terminé";
const REPORT_REGEX = /^\[Rapport du sous-agent "([^"]+)" — (terminé|échoué) — sid:([^\]]+)\]/;

export function isSubagentInjectedMessage(msg: AgentMessage): boolean {
  if (msg.role !== "user") return false;
  return msg.content.startsWith(SUBAGENT_REPORT_PREFIX)
    || msg.content.startsWith(SUBAGENT_SYNTHESIS_PREFIX);
}

export function extractSubagentsFromMessages(messages: AgentMessage[]): SubagentInfo[] {
  const agents: SubagentInfo[] = [];
  for (const msg of messages) {
    if (msg.role !== "user") continue;
    const match = REPORT_REGEX.exec(msg.content);
    if (match) {
      agents.push({
        sessionId: match[3],
        name: match[1],
        type: "explorer",
        status: match[2] === "terminé" ? "completed" : "failed",
        promptPreview: "",
      });
    }
  }
  return agents;
}

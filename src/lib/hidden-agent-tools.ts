const HIDDEN_AGENT_TOOLS = new Set([
  "todo_write",
  "todo_history",
  "todo_pause",
  "todo_resume",
  "agent_diagnostics",
]);

export function isHiddenAgentTool(name: string) {
  return HIDDEN_AGENT_TOOLS.has(name);
}

pub const PARENT_GUIDANCE: &str = "\
# Working with subagents

Use subagents only for independent work that can run in parallel. \
After delegate_task, continue useful independent work without duplicating delegated work. \
Do not repeatedly inspect subagents while they run; terminal reports arrive automatically. \
Use message_subagent only for a concrete correction. \
Use cancel_subagent only for a user request or clearly incorrect direction. \
When no useful independent work remains, give at most one short progress update and \
finish the turn without a tool call.";

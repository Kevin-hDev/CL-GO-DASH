pub const PARENT_GUIDANCE: &str = "\
# Working with subagents

Use subagents only for independent work that can run in parallel. \
After delegate_task, continue useful independent work without duplicating delegated work. \
Do not repeatedly inspect subagents while they run; terminal reports arrive automatically. \
Use message_subagent only for a concrete correction. \
Use cancel_subagent only for a user request or clearly incorrect direction. \
When a coder report includes a pending change, use inspect_subagent_changes to review it. \
If it satisfies the request, use apply_subagent_changes and only report it as integrated after \
that succeeds. If it does not satisfy the request, choose the most appropriate response: request \
a correction, use discard_subagent_changes and fix it yourself, or discard it and delegate new \
work. \
When no useful independent work remains, give at most one short progress update and \
finish the turn without a tool call.";

pub const TODO: &str = "\
# Todo list

For multi-step coding or implementation tasks, use todo_write to keep a short checklist visible.
Call todo_write with the full list when you start planning the work, and update it after each meaningful step.
Use statuses exactly: pending, in_progress, completed. Keep at most one task in_progress.
If you must switch to diagnosis or another task while a checklist is unfinished, call todo_pause first.
Use todo_history to inspect saved checklists and todo_resume to continue one when relevant.
Use todo_delete when a saved checklist is no longer relevant and should not be resumed.
Use agent_diagnostics after a stream interruption or unexplained failure before creating a new diagnostic checklist.
Pass agent_diagnostics limit when you need several recent tool calls; omit it for the latest relevant tool only.
Do not use todo_write for simple questions, single-step edits, or casual conversation.";

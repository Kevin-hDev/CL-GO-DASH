pub const PLAN_MODE: &str = "\
<critical_plan_mode_workflow>
You are in Plan Mode. Follow this workflow exactly and in order.
This Plan Mode workflow overrides the general interactive-choice guidance.

<mandatory_steps>
1. Explore the project with read-only tools when code context is useful.
2. If you need any user answer before publishing the plan, call ask_user_choice before planmode. Do not write Plan Mode questions as normal assistant text.
3. Call planmode only when the final plan is ready.
4. After planmode succeeds, call ask_user_choice for final approval before doing anything else.
5. The final approval question must be exactly: Mettre en oeuvre le plan ?
6. The final approval options must be exactly: Mettre en oeuvre le plan, Continuer a planifier, Quitter le mode plan.
7. If the user chooses Mettre en oeuvre le plan, call exitplanmode with status approved.
8. After exitplanmode approved succeeds, immediately start implementation without waiting for another user message.
9. If the user chooses Continuer a planifier, continue Plan Mode and publish an updated plan.
10. If the user chooses Quitter le mode plan, call exitplanmode with status rejected.
</mandatory_steps>

<allowed_actions>
Use read_file, list_dir, grep, glob, web_search, web_fetch, read-only document/image/spreadsheet tools, agent_diagnostics, ask_user_choice, planmode, and exitplanmode.
</allowed_actions>

<blocked_actions>
Keep the codebase unchanged until exitplanmode approved succeeds. The backend blocks write tools and todo_write while Plan Mode is active.
</blocked_actions>
</critical_plan_mode_workflow>";

#[cfg(test)]
mod tests {
    use super::PLAN_MODE;

    #[test]
    fn plan_prompt_uses_strict_workflow_markers() {
        assert!(PLAN_MODE.contains("<critical_plan_mode_workflow>"));
        assert!(PLAN_MODE.contains("<mandatory_steps>"));
        assert!(PLAN_MODE.contains("<allowed_actions>"));
        assert!(PLAN_MODE.contains("<blocked_actions>"));
        assert!(PLAN_MODE.contains("Follow this workflow exactly and in order"));
        assert!(PLAN_MODE.contains("Do not write Plan Mode questions as normal assistant text"));
        assert!(PLAN_MODE
            .contains("call ask_user_choice for final approval before doing anything else"));
        assert!(PLAN_MODE.contains("Mettre en oeuvre le plan ?"));
    }
}

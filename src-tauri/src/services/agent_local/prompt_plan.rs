pub const PLAN_MODE: &str = "\
<critical_plan_mode_workflow>
You are in Plan Mode. Follow this workflow exactly and in order.
This Plan Mode workflow overrides the general interactive-choice guidance.

<mandatory_steps>
1. Explore the project with read-only tools when code context is useful.
2. If you need an important user answer before publishing the plan, ask it clearly before planmode. You may use normal assistant text or ask_user_choice.
3. Call planmode only when the final plan is ready.
4. planmode asks the user for final approval itself and returns the decision.
5. If planmode returns implement, call exitplanmode with status approved.
6. After exitplanmode approved succeeds, immediately start implementation without waiting for another user message.
7. If planmode returns continue_planning, continue Plan Mode and publish an updated plan.
8. If planmode returns quit_plan, call exitplanmode with status rejected.
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
        assert!(PLAN_MODE.contains("normal assistant text or ask_user_choice"));
        assert!(PLAN_MODE.contains("planmode asks the user for final approval itself"));
        assert!(PLAN_MODE.contains("If planmode returns implement"));
    }
}

use std::path::Path;

#[test]
fn compact_and_detailed_prompts_share_positive_subagent_guidance() {
    let compact = super::prompt_compact::build(Path::new("."), false, None);
    let detailed = super::prompt_detailed::build(Path::new("."), false, None);

    for prompt in [compact, detailed] {
        assert_eq!(prompt.matches("# Working with subagents").count(), 1);
        assert!(prompt.contains("After delegate_task, continue useful independent work"));
        assert!(prompt.contains("Do not repeatedly inspect subagents while they run"));
        assert!(prompt.contains("finish the turn without a tool call"));
    }
}

#[test]
fn dynamic_context_uses_natural_guidance_without_lock_vocabulary() {
    let context = super::subagent_orchestration_context::build_gate_content(&[], false);

    for forbidden in ["Final answer is locked", "Keep the stream active", "blocked"] {
        assert!(!context.contains(forbidden), "forbidden text: {forbidden}");
    }
    assert!(!context.contains("final_answer_allowed"));
    assert!(context.contains("Terminal reports arrive automatically"));
    assert!(context.contains(
        "use it for useful independent work and defer the overall summary"
    ));
    assert!(context.contains("finish this turn without a tool call"));
}

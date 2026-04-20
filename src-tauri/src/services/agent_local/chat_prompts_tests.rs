use crate::services::agent_local::chat_prompts::*;
use crate::services::agent_local::types_ollama::ChatMessage;

fn make_user_msg(text: &str) -> ChatMessage {
    ChatMessage {
        role: "user".to_string(),
        content: text.to_string(),
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
    }
}

const TEST_MODEL_SMALL: &str = "gemma-4-e4b";
const TEST_MODEL_LARGE: &str = "qwen3-32b";

// === prepend_agent_md_context tests ===

#[test]
fn agent_md_prepended_before_system() {
    let mut msgs = vec![make_user_msg("hello")];
    let agent_md = "Be helpful and concise.".to_string();
    prepend_agent_md_context(&mut msgs, Some(agent_md));
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0].role, "system");
    assert!(msgs[0].content.contains("Be helpful and concise."));
}

#[test]
fn agent_md_appended_to_existing_system() {
    let mut msgs = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "Existing system prompt".to_string(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
        },
        make_user_msg("hello"),
    ];
    prepend_agent_md_context(&mut msgs, Some("Agent rules".to_string()));
    assert_eq!(msgs.len(), 2);
    assert!(msgs[0].content.contains("Existing system prompt"));
    assert!(msgs[0].content.contains("Agent rules"));
}

#[test]
fn agent_md_none_does_nothing() {
    let mut msgs = vec![make_user_msg("hello")];
    prepend_agent_md_context(&mut msgs, None);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].role, "user");
}

// === prepare_messages tests ===

#[test]
fn prepare_tool_capable_injects_agent_md() {
    let mut msgs = vec![make_user_msg("hello")];
    let agent_md = Some("Tu réponds en français.".to_string());
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(&mut msgs, wd, true, agent_md, &[], TEST_MODEL_SMALL);
    let sys = &msgs[0];
    assert_eq!(sys.role, "system");
    assert!(sys.content.contains("Tu réponds en français."));
    assert!(sys.content.contains("/tmp/project"));
}

#[test]
fn prepare_tool_capable_injects_tool_prompt() {
    let mut msgs = vec![make_user_msg("hello")];
    let agent_md = Some("Use JSON output.".to_string());
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(&mut msgs, wd, true, agent_md, &[], TEST_MODEL_LARGE);
    let sys = &msgs[0];
    assert!(sys.content.contains("Use JSON output."));
    assert!(sys.content.contains("autonomous"));
}

#[test]
fn prepare_not_tool_capable_no_agent_md() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(&mut msgs, wd, false, None, &[], TEST_MODEL_SMALL);
    let sys = &msgs[0];
    assert!(sys.content.contains("/tmp/project"));
}

#[test]
fn prepare_tool_capable_no_agent_md_file() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(&mut msgs, wd, true, None, &[], TEST_MODEL_SMALL);
    let sys = &msgs[0];
    assert!(sys.content.contains("autonomous"));
    assert!(sys.content.contains("/tmp/project"));
}

#[test]
fn prepare_existing_system_prompt_preserved() {
    let mut msgs = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "Custom system prompt from frontend".to_string(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
        },
        make_user_msg("hello"),
    ];
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(&mut msgs, wd, true, Some("Agent rules".to_string()), &[], TEST_MODEL_LARGE);
    assert_eq!(msgs.len(), 2);
    let sys = &msgs[0];
    assert!(sys.content.contains("Custom system prompt from frontend"));
    assert!(sys.content.contains("Agent rules"));
}

// === skills listing tests ===

#[test]
fn prepare_with_skills_injects_listing() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    let skills = vec![
        ("Test Greeting".to_string(), "Force une salutation".to_string()),
        ("Debug Helper".to_string(), "Aide au debug".to_string()),
    ];
    prepare_messages(&mut msgs, wd, true, None, &skills, TEST_MODEL_SMALL);
    let sys = &msgs[0];
    assert!(sys.content.contains("Test Greeting"));
    assert!(sys.content.contains("Force une salutation"));
    assert!(sys.content.contains("load_skill"));
}

#[test]
fn prepare_without_tools_no_skills() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    let skills = vec![("Test Greeting".to_string(), "Force une salutation".to_string())];
    prepare_messages(&mut msgs, wd, false, None, &skills, TEST_MODEL_SMALL);
    let sys = &msgs[0];
    assert!(!sys.content.contains("Test Greeting"));
}

#[test]
fn prepare_empty_skills_no_section() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    let skills: Vec<(String, String)> = vec![];
    prepare_messages(&mut msgs, wd, true, None, &skills, TEST_MODEL_SMALL);
    let sys = &msgs[0];
    assert!(!sys.content.contains("Available skills"));
}

// === model tier tests ===

#[test]
fn small_model_gets_compact_prompt() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(&mut msgs, wd, true, None, &[], TEST_MODEL_SMALL);
    let sys = &msgs[0];
    assert!(!sys.content.contains("Working with git"));
    assert!(sys.content.contains("autonomous"));
}

#[test]
fn large_model_gets_detailed_prompt() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(&mut msgs, wd, true, None, &[], TEST_MODEL_LARGE);
    let sys = &msgs[0];
    assert!(sys.content.contains("Working with git"));
    assert!(sys.content.contains("Error handling"));
    assert!(sys.content.contains("highly capable"));
}

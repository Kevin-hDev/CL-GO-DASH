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
fn prepare_ollama_tool_capable_injects_agent_md() {
    let mut msgs = vec![make_user_msg("hello")];
    let agent_md = Some("Tu réponds en français.".to_string());
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(&mut msgs, wd, true, agent_md, &[]);
    let sys = &msgs[0];
    assert_eq!(sys.role, "system");
    assert!(sys.content.contains("Tu réponds en français."), "AGENT.md doit être injecté");
    assert!(sys.content.contains("/tmp/project"), "Working dir doit être présent");
}

#[test]
fn prepare_cloud_tool_capable_injects_agent_md_and_tool_prompt() {
    let mut msgs = vec![make_user_msg("hello")];
    let agent_md = Some("Use JSON output.".to_string());
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(&mut msgs, wd, true, agent_md, &[]);
    let sys = &msgs[0];
    assert!(sys.content.contains("Use JSON output."), "AGENT.md injecté");
    assert!(sys.content.contains("tool"), "Tool prompt injecté");
}

#[test]
fn prepare_not_tool_capable_no_agent_md() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(&mut msgs, wd, false, None, &[]);
    let sys = &msgs[0];
    assert!(sys.content.contains("/tmp/project"), "Working dir présent");
    assert!(!sys.content.contains("AGENT.md"), "Pas d'AGENT.md quand non tool_capable");
}

#[test]
fn prepare_tool_capable_no_agent_md_file() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(&mut msgs, wd, true, None, &[]);
    let sys = &msgs[0];
    assert!(sys.content.contains("tool"), "Tool prompt présent");
    assert!(sys.content.contains("/tmp/project"), "Working dir présent");
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
    prepare_messages(&mut msgs, wd, true, Some("Agent rules".to_string()), &[]);
    assert_eq!(msgs.len(), 2, "Pas de message system en double");
    let sys = &msgs[0];
    assert!(sys.content.contains("Custom system prompt from frontend"));
    assert!(sys.content.contains("Agent rules"));
}

// === prepare_messages skills listing tests ===

#[test]
fn prepare_with_skills_injects_listing() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    let skills = vec![
        ("Test Greeting".to_string(), "Force une salutation".to_string()),
        ("Debug Helper".to_string(), "Aide au debug".to_string()),
    ];
    prepare_messages(&mut msgs, wd, true, None, &skills);
    let sys = &msgs[0];
    assert!(sys.content.contains("Test Greeting"), "Skill name présent");
    assert!(sys.content.contains("Force une salutation"), "Skill description présente");
    assert!(sys.content.contains("load_skill"), "Référence au tool load_skill");
}

#[test]
fn prepare_without_tools_no_skills() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    let skills = vec![
        ("Test Greeting".to_string(), "Force une salutation".to_string()),
    ];
    prepare_messages(&mut msgs, wd, false, None, &skills);
    let sys = &msgs[0];
    assert!(!sys.content.contains("Test Greeting"), "Pas de skills quand non tool_capable");
}

#[test]
fn prepare_empty_skills_no_section() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    let skills: Vec<(String, String)> = vec![];
    prepare_messages(&mut msgs, wd, true, None, &skills);
    let sys = &msgs[0];
    assert!(!sys.content.contains("Available skills"), "Pas de section skills si liste vide");
}

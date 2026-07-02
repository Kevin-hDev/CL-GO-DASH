use std::path::Path;

use super::prompt_detailed_sections::{
    CAPABILITIES, CODE, ERRORS, GIT, HONESTY, SAFETY, STYLE, TOOLS, VERIFICATION, WEB_SEARCH,
};

pub fn build(working_dir: &Path, is_git: bool, git_root: Option<&Path>) -> String {
    format!(
        "{IDENTITY}\n\n{CAPABILITIES}\n\n{}\n\n{TOOLS}\n\n{CODE}\n\n{GIT}\n\n{SAFETY}\n\n{ERRORS}\n\n{WEB_SEARCH}\n\n{HONESTY}\n\n{VERIFICATION}\n\n{STYLE}",
        env_section(working_dir, is_git, git_root),
    )
}

const IDENTITY: &str = "\
You are an autonomous coding agent with full access to the user's system through your tools.
You help users with software engineering tasks: writing code, debugging, managing files, \
running commands, searching the web, and more.
You are an agent, not a passive chatbot. You use tools to get things done, \
and you keep the user informed with short visible updates while you work.
You are highly capable and allow users to complete ambitious tasks that would otherwise be \
too complex or take too long.";

fn env_section(working_dir: &Path, is_git: bool, git_root: Option<&Path>) -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let shell = crate::services::env_detect::detect_shell();
    let os_version = crate::services::env_detect::detect_os_version();
    let date = chrono::Local::now().format("%Y-%m-%d");
    let git_flag = if is_git { "true" } else { "false" };
    let git_root_line = match git_root {
        Some(root) if root != working_dir => format!("\n - Git root: {}", root.display()),
        _ => String::new(),
    };
    format!(
        "# Environment\n\
         You have been invoked in the following environment:\n\
         - Primary working directory: {}\n\
         - Is a git repository: {git_flag}{git_root_line}\n\
         - Platform: {os} ({arch})\n\
         - Shell: {shell}\n\
         - OS Version: {os_version}\n\
         - Current date: {date}",
        working_dir.display()
    )
}

pub const OLLAMA_BASE_URL: &str = "http://localhost:11434";

pub mod agent_loop;
pub mod agent_md;
pub mod agent_settings;
pub mod chat_prompts;
pub mod model_size;
pub mod prompt_chat_compact;
pub mod prompt_chat_detailed;
pub mod prompt_compact;
pub mod prompt_detailed;
pub mod modelfile_parser;
pub mod permission_gate;
pub mod security;
pub mod ollama_client;
pub mod ollama_registry;
pub mod ollama_registry_details;
pub mod translation_cache;
pub mod translator;
pub mod ollama_stream;
pub mod session_ops;
pub mod session_store;
pub mod stream_events;
pub mod tab_store;
pub mod tool_definitions;
pub mod tool_dispatcher;
pub mod tool_executor;
pub mod tool_bash;
pub mod tool_files;
pub mod tool_glob;
pub mod tool_grep;
pub mod skill_parser;
pub mod tool_skill_loader;
pub mod tool_web_fetch;
pub mod tool_web_fetch_ip;
#[cfg(test)]
pub mod tool_web_fetch_tests;
#[cfg(test)]
pub mod tool_files_tests;
#[cfg(test)]
pub mod tool_dispatcher_tests;
#[cfg(test)]
pub mod chat_prompts_tests;
#[cfg(test)]
pub mod chat_prompts_chat_tests;
pub mod tool_web_search;
pub mod tool_result_budget;
pub mod types_ollama;
pub mod types_session;
pub mod project_store;
pub mod types_tools;
#[cfg(test)]
pub mod tool_result_budget_tests;

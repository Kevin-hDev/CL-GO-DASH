pub fn ollama_base_url() -> String {
    crate::services::ollama_port::base_url()
}

pub mod agent_loop;
pub mod agent_md;
pub mod agent_settings;
pub mod app_handle_global;
pub mod chat_prompts;
#[cfg(test)]
pub mod chat_prompts_chat_tests;
#[cfg(test)]
pub mod chat_prompts_tests;
pub mod circuit_breaker;
#[cfg(test)]
pub mod circuit_breaker_tests;
pub mod compress_hook;
pub mod eager_dispatch;
pub mod model_size;
pub mod modelfile_parser;
pub mod ollama_client;
pub mod ollama_model_helpers;
pub mod ollama_registry;
pub mod ollama_registry_details;
#[cfg(test)]
mod ollama_registry_tests;
pub mod ollama_stream;
pub mod ollama_stream_process;
pub mod permission_gate;
#[cfg(test)]
pub mod permission_gate_tests;
pub mod project_store;
pub mod prompt_chat_compact;
pub mod prompt_chat_detailed;
pub mod prompt_compact;
pub mod prompt_detailed;
pub mod security;
pub mod session_index;
pub mod session_ops;
pub mod session_store;
pub mod session_subagents;
pub mod skill_parser;
pub mod stream_events;
pub mod subagent_orchestrator;
pub mod subagent_prompts;
#[cfg(test)]
pub mod subagent_prompts_tests;
pub mod subagent_registry;
#[cfg(test)]
pub mod subagent_registry_tests;
pub mod subagent_spawn_channel;
pub mod subagent_task;
#[cfg(test)]
pub mod subagent_task_tests;
pub mod subagent_worktree;
pub mod tab_store;
pub mod tool_bash;
pub mod tool_definitions;
pub mod tool_definitions_forecast;
pub mod tool_definitions_mcp;
pub mod tool_definitions_office;
pub mod tool_definitions_subagent;
pub mod tool_delegate;
pub mod tool_dispatcher;
pub mod tool_dispatcher_forecast;
pub mod tool_dispatcher_forecast_analyze;
pub mod tool_dispatcher_forecast_output;
pub mod tool_dispatcher_mcp;
pub mod tool_dispatcher_office;
#[cfg(test)]
pub mod tool_dispatcher_tests;
pub mod tool_document_read;
#[cfg(test)]
pub mod tool_document_read_tests;
pub mod tool_document_write;
#[cfg(test)]
pub mod tool_document_write_tests;
pub mod tool_document_write_xml;
pub mod tool_executor;
pub mod tool_executor_helpers;
pub mod tool_executor_parallel;
#[cfg(test)]
pub mod tool_executor_parallel_tests;
pub mod tool_files;
#[cfg(test)]
pub mod tool_files_tests;
pub mod tool_glob;
pub mod tool_grep;
pub mod tool_hooks;
#[cfg(test)]
pub mod tool_hooks_tests;
pub mod tool_image_process;
#[cfg(test)]
pub mod tool_image_process_tests;
pub mod tool_image_read;
#[cfg(test)]
pub mod tool_image_read_tests;
pub mod tool_mcp;
pub mod tool_office_utils;
pub mod tool_result_budget;
#[cfg(test)]
pub mod tool_result_budget_tests;
pub mod tool_skill_loader;
pub mod tool_spreadsheet_calamine;
pub mod tool_spreadsheet_read;
#[cfg(test)]
pub mod tool_spreadsheet_read_tests;
pub mod tool_spreadsheet_write;
pub mod tool_spreadsheet_write_edit;
pub mod tool_spreadsheet_write_new;
#[cfg(test)]
pub mod tool_spreadsheet_write_tests;
pub mod tool_validate;
#[cfg(test)]
pub mod tool_validate_tests;
pub mod tool_web_fetch;
pub mod tool_web_fetch_ip;
#[cfg(test)]
pub mod tool_web_fetch_tests;
pub mod tool_web_search;
pub mod translation_cache;
pub mod translator;
pub mod types_ollama;
pub mod types_session;
pub mod types_tools;
pub mod write_guard;
#[cfg(test)]
pub mod write_guard_tests;

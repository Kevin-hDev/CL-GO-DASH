pub mod agent_loop;
pub mod agent_loop_completion;
mod agent_loop_compression;
pub mod agent_loop_errors;
pub mod agent_loop_limits;
mod agent_loop_ollama_request;
pub mod agent_loop_plan;
pub mod agent_loop_support;
pub mod agent_loop_thinking_retry;
pub mod agent_md;
pub mod agent_settings;
pub mod app_handle_global;
pub mod chat_prompts;
#[cfg(test)]
pub mod chat_prompts_chat_tests;
#[cfg(test)]
pub mod chat_prompts_tests;
#[cfg(test)]
pub mod chat_prompts_web_status_tests;
pub mod circuit_breaker;
#[cfg(test)]
pub mod circuit_breaker_tests;
pub mod clone_git;
pub mod clone_git_checks;
pub mod clone_git_cleanup;
pub mod clone_git_link;
pub mod clone_roots;
pub mod clone_session;
pub mod clone_session_build;
pub mod clone_summary;
pub mod clone_summary_ops;
pub mod clone_summary_prompt;
pub mod compress_hook;
pub mod context_budget;
pub mod diagnostic_args;
#[cfg(test)]
mod diagnostic_args_tests;
pub mod diagnostic_redaction;
pub mod disabled_tools_hint;
pub mod eager_dispatch;
pub mod interactive_choice_gate;
pub mod model_customizations;
pub mod model_size;
pub mod modelfile_parser;
pub mod ollama_client;
pub mod ollama_collect;
pub mod ollama_model_helpers;
pub mod ollama_registry;
pub mod ollama_registry_details;
#[cfg(test)]
mod ollama_registry_tests;
pub mod ollama_retry_indicator;
pub mod ollama_stream;
pub mod ollama_stream_process;
#[cfg(test)]
mod ollama_stream_process_tests;
pub mod ollama_stream_request;
pub mod ollama_stream_retry;
pub mod ollama_thinking_retry;
pub mod ollama_tool_parse_retry;
pub mod ollama_tool_role;
#[cfg(test)]
mod ollama_tool_role_tests;
pub mod permission_gate;
#[cfg(test)]
pub mod permission_gate_tests;
pub mod permission_policy;
pub mod plan_mode_controller;
pub mod plan_mode_debug;
pub mod project_store;
pub mod prompt_chat_compact;
pub mod prompt_chat_detailed;
pub mod prompt_compact;
pub mod prompt_detailed;
pub mod prompt_detailed_sections;
pub mod prompt_interactive;
pub mod prompt_plan;
pub mod prompt_todo;
pub mod security;
pub mod sensitive_data;
pub mod session_archive;
pub mod session_family;
pub mod session_id;
pub mod session_index;
pub mod session_locks;
pub mod session_ops;
pub mod session_store;
pub mod session_store_todos;
pub mod session_store_updates;
pub mod session_subagents;
pub mod session_tabs;
pub mod session_tabs_file;
pub mod session_tabs_git;
pub mod session_tabs_state;
pub mod skill_parser;
pub mod stream_buffer;
pub mod stream_diagnostics;
pub mod stream_diagnostics_failure;
pub mod stream_diagnostics_model;
pub mod stream_diagnostics_payload;
pub mod stream_diagnostics_support;
#[cfg(test)]
mod stream_diagnostics_support_tests;
#[cfg(test)]
pub mod stream_diagnostics_tests;
pub mod stream_diagnostics_tools;
pub mod stream_events;
pub mod subagent_activity;
pub mod subagent_completion;
#[cfg(test)]
mod subagent_completion_tests;
pub mod subagent_context;
pub mod subagent_hidden_reports;
pub mod subagent_live_state;
pub mod subagent_orchestration;
pub mod subagent_orchestration_context;
pub mod subagent_panic_supervisor;
#[cfg(test)]
mod subagent_panic_supervisor_tests;
pub mod subagent_profile;
pub mod subagent_prompts;
#[cfg(test)]
pub mod subagent_prompts_tests;
pub mod subagent_queued;
pub mod subagent_registry;
#[cfg(test)]
pub mod subagent_registry_tests;
mod subagent_report_context;
pub mod subagent_spawn_channel;
pub mod subagent_startup_cleanup;
pub mod subagent_status;
pub mod subagent_summary;
pub mod subagent_task;
pub mod subagent_task_stream;
#[cfg(test)]
pub mod subagent_task_tests;
mod subagent_terminal_signal;
pub mod subagent_working_dir;
pub mod subagent_worktree;

pub fn ollama_base_url() -> String {
    crate::services::ollama_port::base_url()
}

pub mod agent_loop;
pub mod agent_loop_completion;
mod agent_loop_compression;
pub mod agent_loop_errors;
pub mod agent_loop_limits;
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
pub mod session_index;
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
pub mod subagent_context;
pub mod subagent_hidden_reports;
pub mod subagent_live_state;
pub mod subagent_orchestration;
pub mod subagent_orchestration_context;
pub mod subagent_profile;
pub mod subagent_prompts;
#[cfg(test)]
pub mod subagent_prompts_tests;
pub mod subagent_queued;
pub mod subagent_registry;
#[cfg(test)]
pub mod subagent_registry_tests;
pub mod subagent_spawn_channel;
pub mod subagent_startup_cleanup;
pub mod subagent_status;
pub mod subagent_summary;
pub mod subagent_task;
pub mod subagent_task_stream;
#[cfg(test)]
pub mod subagent_task_tests;
pub mod subagent_working_dir;
pub mod subagent_worktree;
pub mod tool_bash;
pub mod tool_bash_background;
pub mod tool_bash_changes;
pub mod tool_bash_long;
pub mod tool_catalog;
#[cfg(test)]
pub mod tool_catalog_tests;
pub mod tool_definitions;
pub mod tool_definitions_chat;
pub mod tool_definitions_core;
pub mod tool_definitions_forecast;
pub mod tool_definitions_git;
pub mod tool_definitions_interactive;
pub mod tool_definitions_mcp;
pub mod tool_definitions_office;
pub mod tool_definitions_plan;
pub mod tool_definitions_search;
pub mod tool_definitions_skills;
pub mod tool_definitions_subagent;
pub mod tool_definitions_todo;
pub mod tool_definitions_web;
pub mod tool_delegate;
pub mod tool_delegate_child;
pub mod tool_dispatcher;
pub mod tool_dispatcher_delegate;
pub mod tool_dispatcher_forecast;
pub mod tool_dispatcher_forecast_analyze;
pub mod tool_dispatcher_forecast_models;
pub mod tool_dispatcher_forecast_output;
pub mod tool_dispatcher_forecast_scenario_params;
pub mod tool_dispatcher_mcp;
pub mod tool_dispatcher_office;
#[cfg(test)]
pub mod tool_dispatcher_tests;
#[cfg(test)]
pub mod tool_document_format_tests;
pub mod tool_document_read;
#[cfg(test)]
pub mod tool_document_read_tests;
pub mod tool_document_write;
pub mod tool_document_write_list;
pub mod tool_document_write_numbering;
pub mod tool_document_write_styles;
#[cfg(test)]
pub mod tool_document_write_tests;
pub mod tool_document_write_xml;
pub mod tool_executor;
pub mod tool_executor_compression;
pub mod tool_executor_delegate_batch;
pub mod tool_executor_diagnostics;
pub mod tool_executor_helpers;
pub mod tool_executor_parallel;
pub mod tool_executor_parallel_batch;
pub mod tool_executor_parallel_dispatch;
#[cfg(test)]
pub mod tool_executor_parallel_tests;
pub mod tool_executor_parallel_write;
pub mod tool_executor_plan;
pub mod tool_executor_read_only;
pub mod tool_executor_sequential;
pub mod tool_executor_write;
pub mod tool_files;
#[cfg(test)]
pub mod tool_files_tests;
pub mod tool_glob;
pub mod tool_grep;
pub mod tool_group_catalog;
pub mod tool_hooks;
#[cfg(test)]
pub mod tool_hooks_tests;
pub mod tool_image_process;
#[cfg(test)]
pub mod tool_image_process_limits_tests;
#[cfg(test)]
pub mod tool_image_process_tests;
pub mod tool_image_read;
#[cfg(test)]
pub mod tool_image_read_tests;
pub mod tool_interactive;
pub mod tool_interactive_parse;
#[cfg(test)]
pub mod tool_interactive_tests;
pub mod tool_mcp;
pub mod tool_office_limits;
pub mod tool_office_utils;
pub mod tool_plan;
pub mod tool_plan_approval;
pub mod tool_plan_approval_request;
pub mod tool_plan_guard;
pub mod tool_plan_messages;
pub mod tool_plan_storage;
pub mod tool_prompt_filter;
pub mod tool_result_budget;
#[cfg(test)]
pub mod tool_result_budget_tests;
pub mod tool_result_truncate;
pub mod tool_scan_timeout;
pub mod tool_short_desc;
#[cfg(test)]
pub mod tool_short_desc_tests;
pub mod tool_skill_loader;
pub mod tool_spreadsheet_calamine;
#[cfg(test)]
pub mod tool_spreadsheet_format_tests;
pub mod tool_spreadsheet_read;
#[cfg(test)]
pub mod tool_spreadsheet_read_tests;
pub mod tool_spreadsheet_write;
pub mod tool_spreadsheet_write_edit;
pub mod tool_spreadsheet_write_new;
#[cfg(test)]
pub mod tool_spreadsheet_write_tests;
pub mod tool_subagent_control;
pub mod tool_subagent_format;
pub mod tool_todo;
#[cfg(test)]
mod tool_todo_memory_tests;
pub mod tool_todo_neglect;
pub mod tool_todo_parse;
pub mod tool_todo_state;
pub mod tool_todo_summary;
pub mod tool_validate;
pub mod tool_web_fetch;
pub mod tool_web_fetch_ip;
#[cfg(test)]
pub mod tool_web_fetch_network_tests;
#[cfg(test)]
pub mod tool_web_fetch_tests;
pub mod tool_web_search;
pub mod translation_cache;
pub mod translator;
pub mod types_diagnostics;
pub mod types_interactive;
pub mod types_message;
pub mod types_ollama;
pub mod types_plan;
pub mod types_session;
pub mod types_stream;
pub mod types_todo;
pub mod types_tools;
pub mod web_search_status;
pub mod write_guard;
pub mod write_guard_extract;
#[cfg(test)]
pub mod write_guard_helpers_tests;
pub mod write_guard_registry;
#[cfg(test)]
pub mod write_guard_tests;

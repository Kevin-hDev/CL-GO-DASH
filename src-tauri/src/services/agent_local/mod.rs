pub fn ollama_base_url() -> String {
    crate::services::ollama_port::base_url()
}

include!("agent_local_modules_core.rs");
include!("agent_local_modules_tools.rs");

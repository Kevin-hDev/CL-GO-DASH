use super::types_tools::ToolResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubagentCompletion {
    pub child_session_id: String,
    pub name: String,
    pub subagent_type: String,
    pub status: String,
    pub success: bool,
    pub summary: String,
    pub run_id: Option<String>,
}

impl SubagentCompletion {
    pub fn to_tool_result(&self) -> ToolResult {
        if self.success && self.status == super::subagent_status::COMPLETED {
            return ToolResult::ok(format!(
                "Rapport du sous-agent \"{}\" ({}) :\n\n{}",
                self.name, self.subagent_type, self.summary
            ));
        }

        ToolResult::err(format!(
            "Le sous-agent \"{}\" n'a pas pu terminer correctement. Statut: {}.",
            self.name, self.status
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::SubagentCompletion;

    fn completion(success: bool, status: &str) -> SubagentCompletion {
        SubagentCompletion {
            child_session_id: "child".to_string(),
            name: "Audit".to_string(),
            subagent_type: "explorer".to_string(),
            status: status.to_string(),
            success,
            summary: "Résumé final".to_string(),
            run_id: Some("run".to_string()),
        }
    }

    #[test]
    fn successful_completion_becomes_tool_result_report() {
        let result = completion(true, super::super::subagent_status::COMPLETED).to_tool_result();

        assert!(!result.is_error);
        assert!(result.content.contains("Rapport du sous-agent"));
        assert!(result.content.contains("Résumé final"));
    }

    #[test]
    fn failed_completion_uses_generic_visible_error() {
        let result = completion(false, super::super::subagent_status::FAILED).to_tool_result();

        assert!(result.is_error);
        assert!(result.content.contains("n'a pas pu terminer correctement"));
        assert!(!result.content.contains("Résumé final"));
    }
}

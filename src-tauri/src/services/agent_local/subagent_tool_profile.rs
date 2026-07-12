use serde_json::Value;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SubagentToolProfile {
    Explorer,
    Coder,
}

impl SubagentToolProfile {
    pub fn from_session_type(value: Option<&str>) -> Result<Self, String> {
        match value {
            Some("explorer") => Ok(Self::Explorer),
            Some("coder") => Ok(Self::Coder),
            _ => Err("Profil de sous-agent invalide.".to_string()),
        }
    }

    pub fn tool_names(self, skills_enabled: bool) -> Vec<&'static str> {
        let mut names = match self {
            Self::Explorer => vec![
                "bash",
                "read_file",
                "list_dir",
                "grep",
                "glob",
                "web_search",
                "web_fetch",
            ],
            Self::Coder => vec![
                "bash",
                "read_file",
                "write_file",
                "edit_file",
                "list_dir",
                "grep",
                "glob",
                "web_search",
                "web_fetch",
            ],
        };
        if self == Self::Coder && skills_enabled {
            names.push("load_skill");
        }
        names
    }

    pub fn definitions(self, skills_enabled: bool) -> Vec<Value> {
        let allowed = self.tool_names(skills_enabled);
        super::tool_definitions::get_tool_definitions()
            .into_iter()
            .filter(|definition| {
                definition_name(definition)
                    .is_some_and(|name| allowed.contains(&name))
            })
            .collect()
    }

    #[cfg(test)]
    pub fn definition_names(self, skills_enabled: bool) -> Vec<&'static str> {
        let allowed = self.tool_names(skills_enabled);
        self.definitions(skills_enabled)
            .iter()
            .filter_map(definition_name)
            .filter_map(|name| allowed.iter().copied().find(|allowed| *allowed == name))
            .collect()
    }

    #[cfg(test)]
    pub fn prompt_tool_names(self, skills_enabled: bool) -> Vec<&'static str> {
        self.tool_names(skills_enabled)
    }

    pub fn allows(self, tool_name: &str, skills_enabled: bool) -> bool {
        self.tool_names(skills_enabled).contains(&tool_name)
    }
}

fn definition_name(definition: &Value) -> Option<&str> {
    definition.get("function")?.get("name")?.as_str()
}

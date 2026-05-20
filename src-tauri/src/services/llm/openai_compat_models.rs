use super::types::ModelInfo;

struct StaticModel {
    id: &'static str,
    ctx: u32,
}

const ZAI_MODELS: &[StaticModel] = &[
    StaticModel {
        id: "glm-5.1",
        ctx: 200_000,
    },
    StaticModel {
        id: "glm-5",
        ctx: 200_000,
    },
    StaticModel {
        id: "glm-5-code",
        ctx: 200_000,
    },
    StaticModel {
        id: "glm-4.7",
        ctx: 200_000,
    },
    StaticModel {
        id: "glm-4.6",
        ctx: 200_000,
    },
    StaticModel {
        id: "glm-4.5",
        ctx: 128_000,
    },
    StaticModel {
        id: "glm-4.5v",
        ctx: 128_000,
    },
    StaticModel {
        id: "glm-4.5-air",
        ctx: 128_000,
    },
    StaticModel {
        id: "glm-4.5-flash",
        ctx: 128_000,
    },
];

const XAI_MODELS: &[StaticModel] = &[
    StaticModel {
        id: "grok-4.3",
        ctx: 1_000_000,
    },
    StaticModel {
        id: "grok-4",
        ctx: 256_000,
    },
    StaticModel {
        id: "grok-4-fast-reasoning",
        ctx: 2_000_000,
    },
    StaticModel {
        id: "grok-4-fast-non-reasoning",
        ctx: 2_000_000,
    },
    StaticModel {
        id: "grok-4.20-reasoning",
        ctx: 256_000,
    },
    StaticModel {
        id: "grok-4.20-non-reasoning",
        ctx: 256_000,
    },
    StaticModel {
        id: "grok-3",
        ctx: 131_072,
    },
    StaticModel {
        id: "grok-3-mini",
        ctx: 131_072,
    },
    StaticModel {
        id: "grok-3-fast",
        ctx: 131_072,
    },
    StaticModel {
        id: "grok-2-vision",
        ctx: 32_768,
    },
    StaticModel {
        id: "grok-code-fast",
        ctx: 131_072,
    },
];

pub(super) fn has_static_models(provider_id: &str) -> bool {
    static_models(provider_id).is_some()
}

pub(super) fn static_model_infos(provider_id: &str) -> Option<Vec<ModelInfo>> {
    static_models(provider_id).map(|models| {
        models
            .iter()
            .map(|m| ModelInfo {
                id: m.id.to_string(),
                owned_by: None,
                context_length: Some(m.ctx),
                supports_tools: super::tool_capable::supports_tools(provider_id, m.id),
                supports_vision: super::tool_capable::supports_vision(provider_id, m.id),
                supports_thinking: super::tool_capable::supports_thinking(provider_id, m.id),
                is_free: false,
            })
            .collect()
    })
}

pub(super) fn ping_model(provider_id: &str) -> &'static str {
    match provider_id {
        "zai" => "glm-4.5-flash",
        "xai" => "grok-3-mini",
        _ => "test",
    }
}

fn static_models(provider_id: &str) -> Option<&'static [StaticModel]> {
    match provider_id {
        "zai" => Some(ZAI_MODELS),
        "xai" => Some(XAI_MODELS),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xai_static_models_expose_reasoning_capabilities() {
        let models = static_model_infos("xai").unwrap();
        let grok_mini = models.iter().find(|m| m.id == "grok-3-mini").unwrap();
        let grok_plain = models.iter().find(|m| m.id == "grok-4").unwrap();

        assert!(grok_mini.supports_thinking);
        assert!(!grok_plain.supports_thinking);
    }

    #[test]
    fn zai_static_models_expose_reasoning_capabilities() {
        let models = static_model_infos("zai").unwrap();
        let glm_5 = models.iter().find(|m| m.id == "glm-5").unwrap();
        let glm_46 = models.iter().find(|m| m.id == "glm-4.6").unwrap();
        let glm_flash = models.iter().find(|m| m.id == "glm-4.5-flash").unwrap();

        assert!(glm_5.supports_thinking);
        assert!(glm_46.supports_thinking);
        assert!(glm_flash.supports_thinking);
    }
}

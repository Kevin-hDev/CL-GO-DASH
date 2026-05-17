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
                supports_tools: false,
                supports_vision: false,
                supports_thinking: false,
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

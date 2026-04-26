use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsModel {
    pub name: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub size_vram: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsResponse {
    #[serde(default)]
    pub models: Vec<PsModel>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GpuStatusPayload {
    pub accelerator: String,
    pub vram_used_mb: u64,
    pub vram_total_mb: u64,
    pub model_loaded: Option<String>,
}

pub fn build_gpu_status(
    ps: &PsResponse,
    vram_total_mb: u64,
    vram_used_mb: u64,
) -> GpuStatusPayload {
    match ps.models.first() {
        Some(model) => {
            let accelerator = if model.size_vram > 0 { "GPU" } else { "CPU" };
            GpuStatusPayload {
                accelerator: accelerator.into(),
                vram_used_mb,
                vram_total_mb,
                model_loaded: Some(model.name.clone()),
            }
        }
        None => GpuStatusPayload {
            accelerator: String::new(),
            vram_used_mb: 0,
            vram_total_mb,
            model_loaded: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_with_system_vram() {
        let json = r#"{"models":[{"name":"qwen3:14b","size":9000000000,"size_vram":9000000000}]}"#;
        let ps: PsResponse = serde_json::from_str(json).unwrap();
        let status = build_gpu_status(&ps, 24000, 19000);
        assert_eq!(status.accelerator, "GPU");
        assert_eq!(status.vram_used_mb, 19000);
        assert_eq!(status.vram_total_mb, 24000);
    }

    #[test]
    fn cpu_fallback() {
        let json = r#"{"models":[{"name":"gemma4:e4b","size":3000000000,"size_vram":0}]}"#;
        let ps: PsResponse = serde_json::from_str(json).unwrap();
        let status = build_gpu_status(&ps, 8000, 3000);
        assert_eq!(status.accelerator, "CPU");
    }

    #[test]
    fn empty_returns_idle() {
        let ps = PsResponse { models: vec![] };
        let status = build_gpu_status(&ps, 16000, 0);
        assert!(status.accelerator.is_empty());
        assert!(status.model_loaded.is_none());
    }
}

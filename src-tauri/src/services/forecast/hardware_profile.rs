use serde::Serialize;

const SAFETY_MARGIN_PERCENT: u64 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceFit {
    Comfortable,
    Constrained,
    Insufficient,
    Unknown,
    Cloud,
}

impl ResourceFit {
    pub fn code(self) -> &'static str {
        match self {
            Self::Comfortable => "comfortable",
            Self::Constrained => "constrained",
            Self::Insufficient => "insufficient",
            Self::Unknown => "unknown",
            Self::Cloud => "cloud",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct HardwareProfile {
    pub vram_total_mb: Option<u64>,
    pub vram_available_mb: Option<u64>,
    pub ram_available_mb: Option<u64>,
}

pub fn detect() -> HardwareProfile {
    let vram_total_mb = crate::services::gpu_vram::detect_vram_mb();
    let vram_available_mb = vram_total_mb
        .zip(crate::services::gpu_vram::detect_vram_used_mb())
        .map(|(total, used)| total.saturating_sub(used));
    let mut system = sysinfo::System::new();
    system.refresh_memory();
    let available = system.available_memory() / 1_048_576;
    HardwareProfile {
        vram_total_mb,
        vram_available_mb,
        ram_available_mb: (available > 0).then_some(available),
    }
}

pub fn resource_fit(
    model: &super::catalog::ForecastModelSpec,
    profile: HardwareProfile,
) -> ResourceFit {
    if model.is_cloud {
        return ResourceFit::Cloud;
    }
    let gpu_fit = (model.gpu_supported && model.vram_mb.is_some()).then(|| {
        profile
            .vram_available_mb
            .map_or(ResourceFit::Unknown, |available| {
                fit(model.vram_mb.unwrap_or_default() as u64, available)
            })
    });
    let cpu_fit = model.cpu_supported.then(|| {
        profile
            .ram_available_mb
            .map_or(ResourceFit::Unknown, |available| {
                fit(model.ram_mb as u64, available)
            })
    });
    best_fit(gpu_fit, cpu_fit)
}

pub fn validate_model_resources(model: &super::catalog::ForecastModelSpec) -> Result<(), String> {
    if resource_fit(model, detect()) == ResourceFit::Insufficient {
        return Err("Ressources insuffisantes pour ce modèle".into());
    }
    Ok(())
}

fn fit(required_mb: u64, available_mb: u64) -> ResourceFit {
    if required_mb == 0 {
        return ResourceFit::Comfortable;
    }
    let required_with_margin = required_mb
        .saturating_mul(100 + SAFETY_MARGIN_PERCENT)
        .div_ceil(100);
    if available_mb < required_with_margin {
        ResourceFit::Insufficient
    } else if available_mb >= required_mb.saturating_mul(2) {
        ResourceFit::Comfortable
    } else {
        ResourceFit::Constrained
    }
}

fn best_fit(gpu: Option<ResourceFit>, cpu: Option<ResourceFit>) -> ResourceFit {
    let rank = |value| match value {
        ResourceFit::Comfortable => 3,
        ResourceFit::Constrained => 2,
        ResourceFit::Unknown => 1,
        ResourceFit::Insufficient => 0,
        ResourceFit::Cloud => 4,
    };
    gpu.into_iter()
        .chain(cpu)
        .max_by_key(|value| rank(*value))
        .unwrap_or(ResourceFit::Unknown)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safety_margin_blocks_tight_resources() {
        assert_eq!(fit(1_000, 1_199), ResourceFit::Insufficient);
        assert_eq!(fit(1_000, 1_200), ResourceFit::Constrained);
        assert_eq!(fit(1_000, 2_000), ResourceFit::Comfortable);
    }

    #[test]
    fn unknown_cpu_memory_does_not_reject_a_cpu_capable_model() {
        let model = super::super::catalog::find_model("chronos-bolt-tiny").unwrap();
        let profile = HardwareProfile {
            vram_total_mb: None,
            vram_available_mb: None,
            ram_available_mb: None,
        };

        assert_eq!(resource_fit(model, profile), ResourceFit::Unknown);
    }
}

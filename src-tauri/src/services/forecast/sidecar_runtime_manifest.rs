use std::path::Path;

const MAX_REQUIREMENTS_SIZE: u64 = 16 * 1024;
const MAX_FAMILY_REQUIREMENTS_SIZE: usize = 32 * 1024;

pub(super) fn expected_requirements(sidecar_dir: &Path, family_id: &str) -> Result<String, String> {
    validate_family_id(family_id)?;
    let requirements = sidecar_dir.join("requirements.txt");
    let size = std::fs::metadata(&requirements)
        .map_err(|_| "Runtime Forecast incomplet".to_string())?
        .len();
    if size > MAX_REQUIREMENTS_SIZE {
        return Err("Configuration runtime Forecast invalide".to_string());
    }
    let base = std::fs::read_to_string(requirements)
        .map_err(|_| "Runtime Forecast incomplet".to_string())?;
    let family = family_requirements(family_id)
        .ok_or_else(|| "Adapter Forecast indisponible".to_string())?;
    if family.is_empty() || family.len() > MAX_FAMILY_REQUIREMENTS_SIZE {
        return Err("Configuration runtime Forecast invalide".to_string());
    }
    Ok(format!("{base}\n{family}"))
}

pub(super) fn validate_family_id(family_id: &str) -> Result<(), String> {
    if family_id.is_empty()
        || family_id.len() > 80
        || !family_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
    {
        return Err("Famille Forecast invalide".to_string());
    }
    Ok(())
}

fn family_requirements(family_id: &str) -> Option<&'static str> {
    match family_id {
        "chronos-bolt" | "chronos-2" => Some("pandas<3\nchronos-forecasting==2.2.2\n"),
        "timesfm-2-5" => Some("timesfm\ntransformers\naccelerate\n"),
        "toto-2" => Some(
            "torch>=2.6,<3\ntoto-2 @ git+https://github.com/DataDog/toto.git#subdirectory=toto2\n",
        ),
        "moirai-2" => Some("git+https://github.com/SalesforceAIResearch/uni2ts.git\ngluonts\n"),
        "flowstate" => Some("granite-tsfm\n"),
        "tabpfn-ts" => Some("pandas\ntabpfn-time-series\ntabpfn\n"),
        "tirex" => Some("tirex-ts\n"),
        "kairos" => Some("git+https://github.com/foundation-model-research/Kairos.git\n"),
        "sundial" => Some("transformers>=4.40.1\naccelerate\n"),
        _ => None,
    }
}

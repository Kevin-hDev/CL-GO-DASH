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
        "chronos-bolt" | "chronos-2" => {
            Some("pandas==2.3.3\nchronos-forecasting==2.2.2\n")
        }
        "timesfm-2-5" => Some("timesfm[torch,xreg]==2.0.2\n"),
        "toto-2" => Some("toto-2==2.0.0\n"),
        "moirai-2" => Some(
            "uni2ts==2.0.0\ngluonts==0.14.4\njax[cpu]==0.6.1\nmultiprocess==0.70.16\n",
        ),
        "flowstate" => Some("granite-tsfm==0.3.6\n"),
        "tabpfn-ts" => {
            Some("pandas==2.3.3\ntabpfn-time-series==1.2.0\ntabpfn==8.1.0\n")
        }
        "tirex" => Some("tirex-ts==1.4.2\n"),
        "kairos" => Some(
            "git+https://github.com/foundation-model-research/Kairos.git@0322393840ccf6e2bfe9c663f9dcd088a5a7ee07\n",
        ),
        "sundial" => {
            Some("torch==2.2.2\ntransformers==4.40.1\naccelerate==0.29.3\n")
        }
        _ => None,
    }
}

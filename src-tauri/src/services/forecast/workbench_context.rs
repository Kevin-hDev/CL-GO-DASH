use serde::{Deserialize, Serialize};
use std::sync::{LazyLock, Mutex};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForecastWorkbenchContext {
    pub session_id: String,
    pub analysis_id: Option<String>,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ForecastWorkbenchSnapshot {
    pub context: ForecastWorkbenchContext,
    pub session_name: String,
    pub analysis_name: Option<String>,
}

static ACTIVE_CONTEXT: LazyLock<Mutex<Option<ForecastWorkbenchContext>>> =
    LazyLock::new(|| Mutex::new(None));
const MAX_CONTEXT_NAME_CHARS: usize = 120;

pub async fn set(
    session_id: String,
    analysis_id: Option<String>,
) -> Result<ForecastWorkbenchSnapshot, String> {
    validate_ids(&session_id, analysis_id.as_deref())?;
    let session = crate::services::agent_local::session_store::get(&session_id)
        .await
        .map_err(|_| context_error())?;
    let analysis = load_analysis(&session_id, analysis_id.as_deref()).await?;
    let context = {
        let mut active = ACTIVE_CONTEXT.lock().map_err(|_| context_error())?;
        let next = next_context(active.as_ref(), session_id, analysis_id);
        *active = Some(next.clone());
        next
    };
    Ok(ForecastWorkbenchSnapshot {
        context,
        session_name: bounded_name(session.name),
        analysis_name: analysis.map(|value| bounded_name(value.name)),
    })
}

pub async fn get() -> Result<Option<ForecastWorkbenchSnapshot>, String> {
    let context = ACTIVE_CONTEXT.lock().map_err(|_| context_error())?.clone();
    let Some(context) = context else {
        return Ok(None);
    };
    let session = crate::services::agent_local::session_store::get(&context.session_id)
        .await
        .map_err(|_| context_error())?;
    let analysis = load_analysis(&context.session_id, context.analysis_id.as_deref()).await?;
    Ok(Some(ForecastWorkbenchSnapshot {
        context,
        session_name: bounded_name(session.name),
        analysis_name: analysis.map(|value| bounded_name(value.name)),
    }))
}

async fn load_analysis(
    session_id: &str,
    analysis_id: Option<&str>,
) -> Result<Option<super::types::ForecastResult>, String> {
    let Some(analysis_id) = analysis_id else {
        return Ok(None);
    };
    let analysis = super::storage::load(analysis_id)
        .await
        .map_err(|_| context_error())?;
    if analysis
        .session_id
        .as_deref()
        .is_some_and(|owner| owner != session_id)
    {
        return Err(context_error());
    }
    Ok(Some(analysis))
}

fn next_context(
    current: Option<&ForecastWorkbenchContext>,
    session_id: String,
    analysis_id: Option<String>,
) -> ForecastWorkbenchContext {
    ForecastWorkbenchContext {
        session_id,
        analysis_id,
        revision: current
            .and_then(|value| value.revision.checked_add(1))
            .unwrap_or(1),
    }
}

fn validate_ids(session_id: &str, analysis_id: Option<&str>) -> Result<(), String> {
    crate::services::agent_local::session_store::validate_session_id(session_id)
        .map_err(|_| context_error())?;
    if let Some(id) = analysis_id {
        uuid::Uuid::parse_str(id).map_err(|_| context_error())?;
    }
    Ok(())
}

fn bounded_name(value: String) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .take(MAX_CONTEXT_NAME_CHARS)
        .collect()
}

fn context_error() -> String {
    "Contexte Forecast indisponible".to_string()
}

#[cfg(test)]
mod tests {
    use super::{bounded_name, next_context, validate_ids, ForecastWorkbenchContext};

    const SESSION: &str = "550e8400-e29b-41d4-a716-446655440000";
    const ANALYSIS: &str = "123e4567-e89b-12d3-a456-426614174000";

    #[test]
    fn context_ids_reject_traversal() {
        assert!(validate_ids("../session", Some(ANALYSIS)).is_err());
        assert!(validate_ids(SESSION, Some("../analysis")).is_err());
    }

    #[test]
    fn context_revision_increments_and_keeps_bounded_ids() {
        let first = next_context(None, SESSION.into(), Some(ANALYSIS.into()));
        let second = next_context(Some(&first), SESSION.into(), None);

        assert_eq!(first.revision, 1);
        assert_eq!(second.revision, 2);
        assert_eq!(second.analysis_id, None);
    }

    #[test]
    fn revision_overflow_restarts_without_panicking() {
        let current = ForecastWorkbenchContext {
            session_id: SESSION.into(),
            analysis_id: None,
            revision: u64::MAX,
        };

        assert_eq!(
            next_context(Some(&current), SESSION.into(), None).revision,
            1
        );
    }

    #[test]
    fn context_names_are_sanitized_and_bounded() {
        let name = format!("{}\nsecret", "é".repeat(140));
        let bounded = bounded_name(name);

        assert_eq!(bounded.chars().count(), 120);
        assert!(!bounded.contains('\n'));
    }
}

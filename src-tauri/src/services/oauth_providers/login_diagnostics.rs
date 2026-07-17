use super::{status, ProviderId};

#[derive(Clone)]
pub struct LoginDiagnostic {
    id: uuid::Uuid,
    provider: ProviderId,
    started: std::time::Instant,
}

impl LoginDiagnostic {
    pub fn from_ui(provider: ProviderId, value: &str) -> Result<Self, String> {
        if value.len() != 36 {
            return Err("Connexion impossible".to_string());
        }
        let id = uuid::Uuid::parse_str(value).map_err(|_| "Connexion impossible".to_string())?;
        if id.get_version() != Some(uuid::Version::Random) {
            return Err("Connexion impossible".to_string());
        }
        Ok(Self {
            id,
            provider,
            started: std::time::Instant::now(),
        })
    }

    pub fn stage(&self, stage: &'static str) {
        eprintln!("{} stage={stage}", self.prefix());
    }

    pub fn progress(&self, stage: &'static str, emitted: bool) {
        eprintln!(
            "{} stage=progress_event progress={stage} emitted={emitted}",
            self.prefix()
        );
    }

    pub fn current_state(&self, stage: &'static str) {
        let state = status::connection_evidence(self.provider);
        eprintln!(
            "{} stage={stage} credential_files={} configuration_ready={} invalid_marker={} connected={}",
            self.prefix(),
            state.credential_files,
            state.configuration_ready,
            state.invalid_marker,
            state.connected
        );
    }

    pub fn process_exit(&self, success: bool, code: Option<i32>) {
        eprintln!(
            "{} stage=process_exited success={success} code={}",
            self.prefix(),
            code.map_or_else(|| "signal".to_string(), |value| value.to_string())
        );
    }

    pub fn output(&self, source: &'static str, bytes: usize, ending: &'static str) {
        eprintln!(
            "{} stage=output_finished source={source} bytes={bytes} ending={ending}",
            self.prefix()
        );
    }

    pub fn output_drain(&self, stdout_done: bool, stderr_done: bool, account_error: bool) {
        eprintln!(
            "{} stage=output_drain stdout_done={stdout_done} stderr_done={stderr_done} account_error={account_error}",
            self.prefix()
        );
    }

    fn prefix(&self) -> String {
        format!(
            "[oauth-diagnostic] id={} provider={} elapsed_ms={}",
            self.id,
            self.provider.as_str(),
            self.started.elapsed().as_millis()
        )
    }
}

use super::AcpConnection;
use crate::services::oauth_providers::{
    command_spec, profile_dir, profile_env_names, ProcessKind, ProviderId,
};
use std::path::Path;
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

pub struct AcpProcess {
    _child: Child,
    pub connection: AcpConnection<ChildStdout, ChildStdin>,
}

impl AcpProcess {
    pub async fn spawn(provider: ProviderId, working_dir: &Path) -> Result<Self, String> {
        if !working_dir.is_absolute() || !working_dir.is_dir() {
            return Err("Répertoire ACP invalide".to_string());
        }
        let binary = crate::services::oauth_providers::binary_path(provider)
            .ok_or_else(|| "Client officiel non installé".to_string())?;
        let spec = command_spec(provider, ProcessKind::Acp);
        let home = profile_dir(provider);
        tokio::fs::create_dir_all(&home)
            .await
            .map_err(|_| "Client ACP indisponible".to_string())?;
        let mut command = Command::new(binary);
        for name in profile_env_names(provider) {
            command.env(name, &home);
        }
        let mut child = command
            .args(spec.args)
            .current_dir(working_dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|_| "Client ACP indisponible".to_string())?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Client ACP indisponible".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Client ACP indisponible".to_string())?;
        Ok(Self {
            _child: child,
            connection: AcpConnection::new(stdout, stdin),
        })
    }
}

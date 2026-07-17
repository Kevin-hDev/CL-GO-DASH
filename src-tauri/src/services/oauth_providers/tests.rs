use super::{
    command_spec, credentials_present_in, parse_login_hints, process_environment, profile_dir,
    profile_env_names, remove_credentials_in, ProviderId,
};

#[test]
fn provider_ids_are_strictly_allowlisted() {
    assert_eq!(ProviderId::parse("openai").unwrap(), ProviderId::OpenAi);
    assert_eq!(ProviderId::parse("moonshot").unwrap(), ProviderId::Moonshot);
    assert_eq!(ProviderId::parse("xai").unwrap(), ProviderId::Xai);
    assert!(ProviderId::parse("../moonshot").is_err());
    assert!(ProviderId::parse("google").is_err());
}

#[test]
fn official_login_commands_use_separate_arguments() {
    let kimi = command_spec(ProviderId::Moonshot, super::ProcessKind::Login);
    assert_eq!(kimi.program, "kimi");
    assert_eq!(kimi.args, ["login"]);

    let grok = command_spec(ProviderId::Xai, super::ProcessKind::Login);
    assert_eq!(grok.program, "grok");
    assert_eq!(grok.args, ["login", "--device-auth"]);
}

#[test]
fn isolated_profile_envs_use_the_current_kimi_data_root() {
    assert_eq!(profile_env_names(ProviderId::Moonshot), ["KIMI_CODE_HOME"]);
    assert_eq!(profile_env_names(ProviderId::Xai), ["GROK_HOME"]);
    assert!(profile_env_names(ProviderId::OpenAi).is_empty());
}

#[test]
fn official_clients_cannot_discover_global_agent_resources() {
    for provider in [ProviderId::Moonshot, ProviderId::Xai] {
        let root = profile_dir(provider);
        let environment = process_environment(provider);
        let isolated_home = root.join("agent-home");

        assert!(environment
            .iter()
            .any(|(name, value)| *name == "HOME" && value == &isolated_home));
        assert!(environment
            .iter()
            .any(|(name, value)| *name == "USERPROFILE" && value == &isolated_home));
    }
}

#[test]
fn grok_acp_removes_native_bash_twice() {
    let spec = command_spec(ProviderId::Xai, super::ProcessKind::Acp);
    assert!(spec
        .args
        .windows(2)
        .any(|pair| pair == ["--disallowed-tools", "Bash"]));
    assert!(spec
        .args
        .windows(2)
        .any(|pair| pair == ["--deny", "Bash(*)"]));
    assert!(!spec.args.contains(&"--always-approve"));
    assert!(!spec.args.contains(&"--system-prompt-override"));
}

#[test]
fn isolated_profiles_stay_under_cl_go_data() {
    let root = crate::services::paths::data_dir();
    assert!(profile_dir(ProviderId::Moonshot).starts_with(&root));
    assert!(profile_dir(ProviderId::Xai).starts_with(&root));
    assert_ne!(
        profile_dir(ProviderId::Moonshot),
        profile_dir(ProviderId::Xai)
    );
}

#[test]
fn login_output_keeps_only_bounded_temporary_hints() {
    let raw = "debug token=secret\nOpen https://auth.example/device?access_token=leak and enter ABCD-EFGH\n";
    let hints = parse_login_hints(raw);
    assert_eq!(
        hints.verification_url.as_deref(),
        Some("https://auth.example/device")
    );
    assert_eq!(hints.user_code.as_deref(), Some("ABCD-EFGH"));
    assert!(!hints.verification_url.unwrap().contains("leak"));
}

#[test]
fn login_hints_separate_the_browser_url_and_device_code() {
    let hints = parse_login_hints(
        "Open https://auth.x.ai/device?secret=hidden and enter 45JE-V2VK in the browser",
    );
    assert_eq!(
        hints.verification_url.as_deref(),
        Some("https://auth.x.ai/device")
    );
    assert_eq!(hints.user_code.as_deref(), Some("45JE-V2VK"));
}

#[test]
fn official_credential_files_are_the_connection_source_of_truth() {
    let root = tempfile::tempdir().expect("temporary OAuth profile");
    assert!(!credentials_present_in(root.path(), ProviderId::Moonshot));
    assert!(!credentials_present_in(root.path(), ProviderId::Xai));

    let kimi_credentials = root.path().join("credentials");
    std::fs::create_dir(&kimi_credentials).expect("Kimi credentials directory");
    std::fs::write(kimi_credentials.join("kimi-code.json"), b"credential")
        .expect("Kimi credential metadata");
    assert!(!credentials_present_in(root.path(), ProviderId::Moonshot));
    std::fs::write(
        root.path().join("config.toml"),
        "default_model = \"kimi-code/model\"\n",
    )
    .expect("Kimi model configuration");
    assert!(credentials_present_in(root.path(), ProviderId::Moonshot));

    std::fs::write(root.path().join("auth.json"), b"credential").expect("Grok credential metadata");
    assert!(credentials_present_in(root.path(), ProviderId::Xai));
}

#[tokio::test]
async fn disconnect_removes_only_the_isolated_official_credentials() {
    let root = tempfile::tempdir().expect("temporary OAuth profile");
    let credentials = root.path().join("credentials");
    std::fs::create_dir(&credentials).expect("Kimi credentials directory");
    std::fs::write(credentials.join("kimi-code.json"), b"credential")
        .expect("Kimi credential metadata");
    remove_credentials_in(root.path(), ProviderId::Moonshot)
        .await
        .expect("Kimi disconnect");
    assert!(!credentials.exists());

    std::fs::write(root.path().join("auth.json"), b"credential").expect("Grok credential metadata");
    remove_credentials_in(root.path(), ProviderId::Xai)
        .await
        .expect("Grok disconnect");
    assert!(!root.path().join("auth.json").exists());
}

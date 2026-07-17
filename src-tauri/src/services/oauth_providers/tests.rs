use super::{command_spec, profile_dir, sanitize_login_output, ProviderId};

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
    assert_eq!(kimi.home_env, "KIMI_CODE_HOME");

    let grok = command_spec(ProviderId::Xai, super::ProcessKind::Login);
    assert_eq!(grok.program, "grok");
    assert_eq!(grok.args, ["login", "--device-auth"]);
    assert_eq!(grok.home_env, "GROK_HOME");
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
    let clean = sanitize_login_output(raw);
    assert!(clean.contains("https://auth.example/device"));
    assert!(clean.contains("ABCD-EFGH"));
    assert!(!clean.contains("secret"));
    assert!(!clean.contains("leak"));
    assert!(clean.len() <= 512);
}

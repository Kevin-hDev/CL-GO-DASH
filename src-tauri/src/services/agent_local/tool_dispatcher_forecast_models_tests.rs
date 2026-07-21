use super::auto_candidates;

fn model(id: &str, runnable: bool, cloud: bool) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "runnable": runnable,
        "is_cloud": cloud
    })
}

#[test]
fn auto_candidates_are_bounded_to_five() {
    let models = (0..8)
        .map(|index| model(&format!("model-{index}"), true, false))
        .collect::<Vec<_>>();

    assert_eq!(auto_candidates(&models, false).len(), 5);
}

#[test]
fn auto_excludes_cloud_by_default() {
    let models = [model("local", true, false), model("cloud", true, true)];
    let candidates = auto_candidates(&models, false);

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0]["model_id"], "local");
}

#[test]
fn auto_excludes_models_without_a_runnable_adapter() {
    let models = [model("ready", true, false), model("missing", false, false)];
    let candidates = auto_candidates(&models, true);

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0]["model_id"], "ready");
}

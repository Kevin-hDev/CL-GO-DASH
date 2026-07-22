use super::runnable_state;
use crate::services::forecast::{catalog, registry};

#[test]
fn local_model_is_not_runnable_until_its_runtime_is_ready() {
    let model = catalog::find_model("chronos-bolt-tiny").unwrap();
    let runtime = registry::find_runtime(model.id);

    assert!(!runnable_state(model, runtime, true, false, false));
    assert!(runnable_state(model, runtime, true, false, true));
}

#[test]
fn cloud_model_still_depends_on_provider_configuration() {
    let model = catalog::find_model("timegpt-2-mini").unwrap();
    let runtime = registry::find_runtime(model.id);

    assert!(!runnable_state(model, runtime, false, false, false));
    assert!(runnable_state(model, runtime, false, true, true));
}

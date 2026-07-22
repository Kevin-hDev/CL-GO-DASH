use super::*;

#[test]
fn auto_models_schema_requires_a_bounded_profile_id() {
    let auto = forecast_models_definition_for(true);
    let manual = forecast_models_definition_for(false);
    let auto_parameters = &auto["function"]["parameters"];

    assert_eq!(
        auto_parameters["properties"]["data_profile_id"]["maxLength"],
        64
    );
    assert!(auto_parameters["required"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "data_profile_id"));
    assert!(manual["function"]["parameters"]["required"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn auto_models_schema_accepts_an_explicit_user_model() {
    let auto = forecast_models_definition_for(true);
    let requested = &auto["function"]["parameters"]["properties"]["requested_model_id"];

    assert_eq!(requested["type"], "string");
    assert_eq!(
        requested["maxLength"],
        crate::services::forecast::limits::MAX_MODEL_ID_CHARS
    );
}

#[test]
fn model_tool_explains_that_candidates_are_already_confidence_safe() {
    let auto = forecast_models_definition_for(true);
    let description = auto["function"]["description"].as_str().unwrap();

    assert!(description.contains("confidence"));
    assert!(description.contains("candidates"));
}

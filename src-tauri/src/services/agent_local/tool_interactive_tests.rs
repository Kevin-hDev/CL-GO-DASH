use serde_json::json;

use super::tool_interactive_parse::{parse_questions, validate_answers};
use super::types_interactive::AgentInteractiveAnswer;

fn valid_args() -> serde_json::Value {
    json!({
        "questions": [{
            "header": "Plan",
            "question": "Quelle suite choisir ?",
            "options": [
                {"label": "Rapide", "description": "Faire le minimum", "recommended": true},
                {"label": "Complet", "description": "Faire toute la passe"}
            ]
        }]
    })
}

#[test]
fn parse_accepts_valid_choice_request() {
    let questions = parse_questions(&valid_args()).unwrap();

    assert_eq!(questions.len(), 1);
    assert_eq!(questions[0].options.len(), 2);
    assert!(questions[0].options[0].recommended);
}

#[test]
fn parse_rejects_more_than_four_questions() {
    let questions: Vec<_> = (0..5)
        .map(|index| {
            json!({
                "header": format!("Q{index}"),
                "question": "Choisir ?",
                "options": [
                    {"label": "A", "description": "A"},
                    {"label": "B", "description": "B"}
                ]
            })
        })
        .collect();

    assert!(parse_questions(&json!({ "questions": questions })).is_err());
}

#[test]
fn parse_rejects_invalid_option_count() {
    let err = parse_questions(&json!({
        "questions": [{
            "header": "Plan",
            "question": "Choisir ?",
            "options": [{"label": "A", "description": "A"}]
        }]
    }))
    .unwrap_err();

    assert!(err.contains("2 à 4"));
}

#[test]
fn parse_rejects_multiple_recommended_options() {
    let err = parse_questions(&json!({
        "questions": [{
            "header": "Plan",
            "question": "Choisir ?",
            "options": [
                {"label": "A", "description": "A", "recommended": true},
                {"label": "B", "description": "B", "recommended": true}
            ]
        }]
    }))
    .unwrap_err();

    assert!(err.contains("une seule"));
}

#[test]
fn validate_answers_rejects_unknown_label() {
    let questions = parse_questions(&valid_args()).unwrap();
    let err = validate_answers(
        &questions,
        vec![AgentInteractiveAnswer {
            question_index: 0,
            selected_labels: vec!["Inconnu".into()],
            custom_answer: None,
        }],
    )
    .unwrap_err();

    assert!(err.contains("inconnu"));
}

#[tokio::test]
async fn pending_store_is_bounded_for_tests() {
    super::interactive_choice_gate::fill_pending_for_test(64).await;

    assert_eq!(
        super::interactive_choice_gate::pending_len_for_test().await,
        64
    );
    super::interactive_choice_gate::clear_pending_for_test().await;
}

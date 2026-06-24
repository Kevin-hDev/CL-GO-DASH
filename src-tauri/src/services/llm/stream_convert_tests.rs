use super::*;
use crate::services::llm::vision;

fn user_with_png() -> ChatMessage {
    ChatMessage {
        role: "user".into(),
        content: "Regarde".into(),
        images: Some(vec!["iVBORw0KGgo=".into()]),
        ..Default::default()
    }
}

#[test]
fn openai_style_image_uses_object_url() {
    let out = message_to_openai(&user_with_png(), "openai");
    assert_eq!(out["role"], "user");
    assert_eq!(out["content"][0]["type"], "text");
    assert_eq!(out["content"][1]["type"], "image_url");
    assert_eq!(
        out["content"][1]["image_url"]["url"],
        "data:image/png;base64,iVBORw0KGgo="
    );
}

#[test]
fn mistral_image_uses_string_url() {
    let out = message_to_openai(&user_with_png(), "mistral");
    assert_eq!(out["content"][1]["type"], "image_url");
    assert_eq!(
        out["content"][1]["image_url"],
        "data:image/png;base64,iVBORw0KGgo="
    );
}

#[test]
fn strip_images_adds_user_note() {
    let mut msgs = vec![user_with_png()];
    vision::strip_images(&mut msgs);
    assert!(msgs[0].images.is_none());
    assert!(msgs[0].content.contains("does not support vision"));
}

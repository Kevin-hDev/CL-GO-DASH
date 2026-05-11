use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TgResponse<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub description: Option<String>,
    pub error_code: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct TgUpdate {
    pub update_id: i64,
    pub message: Option<TgMessage>,
}

#[derive(Debug, Deserialize)]
pub struct TgMessage {
    pub message_id: i64,
    pub from: Option<TgUser>,
    pub chat: TgChat,
    pub text: Option<String>,
    pub entities: Option<Vec<TgEntity>>,
}

#[derive(Debug, Deserialize)]
pub struct TgUser {
    pub id: i64,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TgChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
}

#[derive(Debug, Deserialize)]
pub struct TgEntity {
    #[serde(rename = "type")]
    pub entity_type: String,
    pub offset: u32,
    pub length: u32,
}

#[derive(Debug, Deserialize)]
pub struct TgSentMessage {
    pub message_id: i64,
}

impl TgChat {
    pub fn is_group(&self) -> bool {
        self.chat_type == "group" || self.chat_type == "supergroup"
    }
}

impl TgMessage {
    pub fn has_bot_mention(&self, bot_username: &str) -> bool {
        let Some(entities) = &self.entities else {
            return false;
        };
        let Some(text) = &self.text else { return false };
        let lower_bot = bot_username.to_lowercase();
        for e in entities {
            if e.entity_type == "mention" {
                let start = e.offset as usize;
                let end = start + e.length as usize;
                if let Some(mention) = text.get(start..end) {
                    let clean = mention.trim_start_matches('@').to_lowercase();
                    if clean == lower_bot {
                        return true;
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_detection() {
        let chat = TgChat {
            id: 1,
            chat_type: "group".into(),
        };
        assert!(chat.is_group());
        let dm = TgChat {
            id: 2,
            chat_type: "private".into(),
        };
        assert!(!dm.is_group());
        let super_g = TgChat {
            id: 3,
            chat_type: "supergroup".into(),
        };
        assert!(super_g.is_group());
    }

    #[test]
    fn bot_mention_detection() {
        let msg = TgMessage {
            message_id: 1,
            from: None,
            chat: TgChat {
                id: 1,
                chat_type: "group".into(),
            },
            text: Some("@MyBot hello".into()),
            entities: Some(vec![TgEntity {
                entity_type: "mention".into(),
                offset: 0,
                length: 6,
            }]),
        };
        assert!(msg.has_bot_mention("MyBot"));
        assert!(msg.has_bot_mention("mybot"));
        assert!(!msg.has_bot_mention("OtherBot"));
    }
}

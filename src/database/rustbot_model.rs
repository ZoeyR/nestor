use super::models::{Factoid, FactoidEnum, NewFactoid};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RFactoid {
    pub key: String,
    pub val: RFactoidValue,
}

#[derive(Serialize, Deserialize)]
pub struct RFactoidValue {
    pub intent: String,
    pub message: String,
    pub editor: RFactoidEditor,
    pub time: NaiveDateTime,
    pub frozen: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RFactoidEditor {
    pub nickname: String,
    pub username: String,
    pub hostname: String,
}

impl From<Factoid> for RFactoid {
    fn from(factoid: Factoid) -> Self {
        RFactoid {
            key: factoid.label,
            val: RFactoidValue {
                intent: factoid.intent.to_string(),
                message: factoid.description,
                editor: RFactoidEditor {
                    nickname: factoid.nickname,
                    username: "".into(),
                    hostname: "".into(),
                },
                time: factoid.timestamp,
                frozen: factoid.locked,
            },
        }
    }
}

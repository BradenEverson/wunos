use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::filters::ws::Message;

use crate::game::card::Card;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynMessage {
    sender: Option<Uuid>,
    action: Action
}

impl DynMessage {
    pub fn new_msg(from: Uuid, action: Action) -> Self {
        Self { sender: Some(from), action }
    }

    pub fn broadcast(text: &str) -> Self {
        Self { sender: None, action: Action::Message(text.into()) }
    }

    pub fn draw(player: Uuid) -> Self {
        Self { sender: Some(player), action: Action::Win(player) }
    }
}

impl Into<Message> for DynMessage {
    fn into(self) -> Message {
        let text = serde_json::to_string(&self).unwrap();
        Message::text(text)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Message(String),
    PlayCard(Card),
    Win(Uuid),
    DrawCard,
    DrawnCard(Card)
}


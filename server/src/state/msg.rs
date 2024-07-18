use serde::{Deserialize, Serialize};
use warp::filters::ws::Message;

use crate::game::card::Card;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynMessage {
    sender: Option<String>,
    action: Action
}

impl DynMessage {
    pub fn new_msg(from: String, action: Action) -> Self {
        Self { sender: Some(from), action }
    }

    pub fn broadcast(text: &str) -> Self {
        Self { sender: None, action: Action::Message(text.into()) }
    }

    pub fn draw(drawn: Card) -> Self {
        Self { sender: None, action: Action::DrawnCard(drawn) }
    }
    pub fn top_card(top: Card) -> Self {
        Self { sender: None, action: Action::TopCard(top) }
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
    Win,
    DrawCard,
    DrawnCard(Card),
    Start,
    TopCard(Card),
    SetName(String)
}


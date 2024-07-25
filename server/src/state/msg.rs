use serde::{Deserialize, Serialize};
use warp::filters::ws::Message;

use crate::game::card::Card;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynMessage {
    pub sender: Option<String>,
    pub action: Action
}

impl DynMessage {
    pub fn new_msg(from: Option<String>, action: Action) -> Self {
        Self { sender: from, action }
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

impl From<DynMessage> for Message {
    fn from(val: DynMessage) -> Self {
        let text = serde_json::to_string(&val).unwrap();
        Message::text(text)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Message(String),
    PlayCard(Card),
    AcceptPlayCard,
    DenyPlayCard,
    Win,
    DrawCard,
    DrawnCard(Card),
    Start,
    Started([Card; 7]),
    TopCard(Card),
    SetName(String),
    YourTurn,
}


use uuid::Uuid;

use crate::game::card::Card;

pub struct DynMessage {
    sender: Uuid,
    action: Action
}

pub enum Action {
    Message(String),
    PlayCard(Card),
    DrawCard
}


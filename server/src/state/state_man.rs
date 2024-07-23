
use std::collections::HashMap;

use uuid::Uuid;

use crate::{game::deck::Deck, res::err::Result};

use super::{msg::DynMessage, player::Player};


#[derive(Default)]
pub struct GameState {
    pub in_game: bool,
    pub turn: Uuid,
    pub deck: Deck,
    pub players: HashMap<Uuid, Player>,        
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_player(&mut self, id: Uuid, player: &mut Player) {
        if self.num_players() == 0 {
            player.set_admin();
        }

        self.players.insert(id, player.clone());
    }

    pub fn num_players(&self) -> usize {
        self.players.len()
    }

    pub fn broadcast(&self, msg: DynMessage) -> Result<()> {
        self.broadcast_but(msg, &[])    
    }

    pub fn broadcast_but(&self, msg: DynMessage, without: &[Uuid]) -> Result<()> {
        let connections = &self.players;

        for (_, player) in connections.iter().filter(|(id, _)| !without.contains(id)) {
            player.send_msg(&msg)?;
        }

        Ok(())
    }
}

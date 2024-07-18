use std::{collections::HashMap, sync::Mutex};

use uuid::Uuid;

use crate::{game::deck::Deck, res::err::Result};

use super::{msg::DynMessage, player::Player};


#[derive(Default)]
pub struct GameState {
    pub in_game: bool,
    pub turn: Uuid,
    pub deck: Mutex<Deck>,
    pub players: Mutex<HashMap<Uuid, Player>>,        
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_player(&mut self, id: Uuid, player: &mut Player) {
        if self.num_players() == 0 {
            player.set_admin();
        }

        self.players.lock().unwrap().insert(id, player.clone());
    }

    pub fn num_players(&self) -> usize {
        self.players.lock().expect("Poisoned Mutex :(").len()
    }

    pub fn broadcast(&self, msg: DynMessage) -> Result<()> {
        let connections = self.players.lock().expect("Poisoned Mutex :(");

        for (_, player) in connections.iter() {
            player.send_msg(&msg)?;
        }

        Ok(())
    }
}

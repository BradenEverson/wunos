use std::{collections::HashMap, sync::Mutex};

use uuid::Uuid;

use crate::res::err::Result;

use super::player::Player;


#[derive(Default)]
pub struct GameState {
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

    pub fn broadcast(&self, msg: warp::ws::Message) -> Result<()> {
        let connections = self.players.lock().expect("Poisoned Mutex :(");

        for (_, player) in connections.iter() {
            player.send_msg(&msg)?;
        }

        Ok(())
    }
}

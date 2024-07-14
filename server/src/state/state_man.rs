use std::sync::Mutex;

use crate::res::err::Result;

use super::player::Player;


#[derive(Default)]
pub struct GameState {
    pub players: Mutex<Vec<Player>>,        
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn broadcast(&self, msg: warp::ws::Message) -> Result<()> {
        let connections = self.players.lock().expect("Poisoned Mutex :(");

        for player in connections.iter() {
            player.send_msg(&msg)?;
        }

        Ok(())
    }
}

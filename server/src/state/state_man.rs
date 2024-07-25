
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
    direction: Direction
}

#[derive(Default)]
pub enum Direction {
    #[default]
    Forward,
    Backward
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reverse(&mut self) {
        self.direction = match &self.direction {
            Direction::Forward => Direction::Backward,
            Direction::Backward => Direction::Forward
        }
    }

    pub fn after(&self, curr: &Uuid) -> Option<Uuid> {
        if !self.players.contains_key(curr) {
            return None
        }
        match self.direction {
            Direction::Forward => {
            
                let mut key_cycle = self.players.keys().cycle();

                loop {
                    let key_cmp = key_cycle.next().unwrap();
                    if key_cmp == curr {
                        return Some(*key_cycle.next().unwrap());
                    }
                }
            }
            Direction::Backward => {
                let mut key_cycle: Vec<Uuid> = self.players.clone().keys().map(|key| *key).collect();
                key_cycle.reverse();

                let mut key_cycle = key_cycle.iter().cycle();

                loop {
                    let key_cmp = key_cycle.next().unwrap();
                    if key_cmp == curr {
                        return Some(*key_cycle.next().unwrap());
                    }
                }
            }
        };
    }

    pub fn send_msg(&mut self, player_id: &Uuid, msg: &DynMessage) -> Result<()> {
        self.players[player_id].send_msg(&msg)
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

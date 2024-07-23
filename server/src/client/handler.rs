use std::sync::{Arc, RwLock};

use futures_util::{stream::StreamExt, SinkExt};
use tokio::sync::mpsc;
use uuid::Uuid;
use warp::filters::ws::Message;

use crate::state::{msg::{Action, DynMessage}, player::{Player, Role}, state_man::GameState};

pub async fn handle_connection(ws: warp::ws::WebSocket, state: Arc<RwLock<GameState>>) {
    let (mut sender, mut receiver) = ws.split();

    let (tx, mut rx) = mpsc::unbounded_channel();

    let player_id = Uuid::new_v4();
    let mut player_name: Option<String> = None;

    let mut player = Player::new(tx.clone());

    if state.read().unwrap().num_players() == 0 {
        player.set_admin();
        player.send_msg(&DynMessage::broadcast("You're admin! Please type START to start the game when you'd like")).expect("Message send error");
    }

    state.write().unwrap().players.insert(player_id, player);

    tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(msg) => {
                    if let Ok(text) = msg.to_str() {
                        if let Ok(action) = serde_json::from_str::<Action>(text) {
                            match action {
                                Action::Message(txt) => {
                                    // Broadcast message from user to everyone else
                                    if let Some(name) = &player_name {
                                        let msg = DynMessage::new_msg(name.to_string(), Action::Message(txt.to_string()));

                                        state.read().unwrap().broadcast_but(msg, &[player_id]).expect("Error broadcasting");
                                    }
                                },
                                Action::Start => { 
                                    // Double check they are admin, if so start game
                                    if state.read().unwrap().in_game || state.read().unwrap().players[&player_id].role != Role::Admin {
                                        continue;
                                    }

                                    {
                                        let mut write_state = state.write().unwrap();
                                        write_state.in_game = true;
                                        write_state.turn = player_id;
                                    }

                                    let curr_card = {
                                        let mut write_state = state.write().unwrap();

                                        write_state.deck.start_game();
                                        *write_state.deck.get_facing().unwrap()
                                    };

                                    state.read().unwrap().broadcast(DynMessage::top_card(curr_card)).expect("Broadcast Message Failure");
                                },
                                Action::Win => {
                                    // Check game exists, then what players hand size is
                                    if !state.read().unwrap().in_game {
                                        continue;
                                    }
                                },
                                Action::DrawCard => {
                                    // Draw card for user and send it back as a drawn card
                                    if !state.read().unwrap().in_game || state.read().unwrap().turn != player_id {
                                        continue;
                                    }
                                },
                                Action::PlayCard(card) => {
                                    // Check if card can be played on top of current deck, if so do
                                    // it and return a success. If not then return a failure
                                    if !state.read().unwrap().in_game || state.read().unwrap().turn != player_id {
                                        continue;
                                    }
                                    
                                },
                                Action::SetName(name) => {
                                    // Set user's name to `name`
                                    state.write().unwrap().players.get_mut(&player_id).unwrap().set_name(&name);
                                    player_name = Some(name);
                                },
                                Action::DrawnCard(_) => { unreachable!("User will never initialize a DrawnCard action") },
                                Action::TopCard(_) => { unreachable!("User will never call TopCard") },

                            }
                        }
                    }
                }, 
                Err(e) => {
                    eprintln!("Message handling error: {}", e);
                    break;
                }
            }
        }

        // Remove connection on disconect
        state.write().unwrap().players.remove(&player_id);
    });

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let entry = Message::text("Enter your name: ".to_string());
    tx.send(entry).unwrap();
}

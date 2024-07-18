use std::sync::Arc;

use futures_util::{stream::StreamExt, SinkExt};
use tokio::sync::mpsc;
use uuid::Uuid;
use warp::filters::ws::Message;

use crate::state::{msg::{Action, DynMessage}, player::{Player, Role}, state_man::GameState};

pub async fn handle_connection(ws: warp::ws::WebSocket, state: Arc<GameState>) {
    let (mut sender, mut receiver) = ws.split();

    let (tx, mut rx) = mpsc::unbounded_channel();

    let player_id = Uuid::new_v4();

    let mut player = Player::new(tx.clone());

    if state.in_game {

    }

    if state.num_players() == 0 {
        player.set_admin();
        player.send_msg(&DynMessage::broadcast("You're admin! Please type START to start the game when you'd like")).expect("Message send error");
    }

    state.players.lock().unwrap().insert(player_id, player);

    tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(msg) => {
                    if let Ok(text) = msg.to_str() {
                        if let Ok(action) = serde_json::from_str::<Action>(text) {
                            match action {
                                Action::Message(txt) => {
                                    // Broadcast message from user to everyone else
                                },
                                Action::Start => { 
                                    // Double check they are admin, if so start game
                                    if state.in_game {
                                        continue;
                                    }

                                    //state.turn = player_id;
                                    //state.in_game = true;

                                    let curr_card = {
                                        state.deck.lock().unwrap().start_game();
                                        state.deck.lock().unwrap().get_facing().unwrap().clone()
                                    };

                                    state.broadcast(DynMessage::top_card(curr_card)).expect("Broadcast Message Failure");
                                },
                                Action::Win => {
                                    // Check game exists, then what players hand size is
                                    if !state.in_game {
                                        continue;
                                    }
                                },
                                Action::DrawCard => {
                                    // Draw card for user and send it back as a drawn card
                                    if !state.in_game || state.turn != player_id {
                                        continue;
                                    }
                                },
                                Action::PlayCard(card) => {
                                    // Check if card can be played on top of current deck, if so do
                                    // it and return a success. If not then return a failure
                                    if !state.in_game || state.turn != player_id {
                                        continue;
                                    }
                                    
                                },
                                Action::SetName(name) => {
                                    // Set user's name to `name`
                                    { state.players.lock().unwrap().get_mut(&player_id).unwrap().set_name(&name) };
                                },
                                Action::DrawnCard(_) => { unreachable!("User will never initialize a DrawnCard action") },
                                Action::TopCard(_) => { unreachable!("User will never call TopCard") },

                            }
                        }
                        /*let player_name = {
                            let mut state_players = state.players.lock().unwrap();
                            let player = state_players.get_mut(&player_id).unwrap();

                            if let Some(name) = player.get_name() {
                                Some(name.to_string())
                            } else {
                                player.set_name(text.trim())
                            }
                        };

                        if let Some(_) = player_name {
                            match text.trim() {
                                "START" => {
                                    let role = {
                                        state.players.lock().unwrap().get(&player_id).unwrap().role.clone()
                                    };

                                    match role {
                                        Role::Admin => {println!("Admin wants us to start")},
                                        Role::User => {}
                                    }
                                },
                                _ => {
                                    let broadcast_msg = DynMessage::new_msg(player_id, Action::Message(text.to_string()));

                                    if let Err(e) = state.broadcast(broadcast_msg) {
                                        eprintln!("Message broadcast error: {}", e);
                                        break;
                                    }
                                }
                            }

                        } else {
                            state.broadcast(DynMessage::broadcast(&format!("{} has joined the party", text.trim()))).expect("Failed to send welcome message to all clients");
                        }*/
                    }
                }, 
                Err(e) => {
                    eprintln!("Message handling error: {}", e);
                    break;
                }
            }
        }

        // Remove connection on disconect
        state.players.lock().unwrap().remove(&player_id);
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

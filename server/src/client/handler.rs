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

                        let player_name = {
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

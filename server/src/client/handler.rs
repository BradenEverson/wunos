use std::{borrow::BorrowMut, sync::Arc};

use futures_util::{stream::StreamExt, SinkExt};
use tokio::sync::mpsc;
use uuid::Uuid;
use warp::filters::ws::Message;

use crate::state::{player::Player, state_man::GameState};

pub async fn handle_connection(ws: warp::ws::WebSocket, state: Arc<GameState>) {
    let (mut sender, mut receiver) = ws.split();

    let (tx, mut rx) = mpsc::unbounded_channel();

    let player_id = Uuid::new_v4();

    state.players.lock().unwrap().insert(player_id, Player::new(tx.clone()));

    tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(msg) => {
                    if let Ok(text) = msg.to_str() {
                        let mut state_players = state.players.lock().unwrap();
                        let player = state_players.borrow_mut().get_mut(&player_id).unwrap();

                        if let Some(name) = player.get_name() {
                            let broadcast_msg = Message::text(format!("{}: {}", name, text));

                            if let Err(e) = state.broadcast(broadcast_msg) {
                                eprintln!("Message broadcast error: {}", e);
                                break;
                            }
                        } else {
                            player.set_name(text.trim());
                            //player.send_msg(&Message::text(format!("Welcome, {}!", text.trim()))).expect("Failed to send message to client");
                            state.broadcast(Message::text(format!("{} has joined the party", text.trim()))).expect("Failed to send welcome message to all clients");
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

    let entry = Message::text(format!("Welcome new player, please enter your name:"));
    tx.send(entry).unwrap();
}
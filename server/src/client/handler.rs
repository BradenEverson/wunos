use std::sync::Arc;

use futures_util::{stream::StreamExt, SinkExt};
use tokio::sync::mpsc;
use warp::filters::ws::Message;

use crate::state::{player::Player, state_man::GameState};

pub async fn handle_connection(ws: warp::ws::WebSocket, state: Arc<GameState>) {
    let (mut sender, mut receiver) = ws.split();

    let (tx, mut rx) = mpsc::unbounded_channel();

    let player = Player::new(tx.clone());

    state.players.lock().unwrap().push(player.clone());

    tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(msg) => {
                    if let Ok(text) = msg.to_str() {
                        let broadcast_msg = Message::text(format!("Player: {}", text));

                        if let Err(e) = state.broadcast(broadcast_msg) {
                            eprintln!("Message broadcast error: {}", e);
                            break;
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
        state.players.lock().unwrap().retain(|connection| connection != &player);
    });

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let entry = Message::text(format!("Welcome new player!"));
    tx.send(entry).unwrap();
}

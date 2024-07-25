use std::sync::{Arc, RwLock};

use futures_util::{stream::StreamExt, SinkExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{game::{card::{Card, Color}, deck::Deck}, state::{msg::{Action, DynMessage}, player::{Player, Role}, state_man::GameState}};

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
                                        let msg = DynMessage::new_msg(Some(name.to_string()), Action::Message(txt.to_string()));

                                        state.read().unwrap().broadcast_but(msg, &[player_id]).expect("Error broadcasting");
                                    }
                                },
                                Action::Start => { 
                                    // Double check they are admin, if so start game
                                    if state.read().unwrap().in_game || state.read().unwrap().players[&player_id].role != Role::Admin {
                                        continue;
                                    }
                                    println!("Game start time");

                                    {
                                        let mut write_state = state.write().unwrap();
                                        write_state.in_game = true;
                                        write_state.turn = player_id;
                                    }

                                    let num_players = {
                                        state.read().unwrap().num_players().clone()
                                    };

                                    let mut hands: Vec<[Card; 7]> = vec![];

                                    {
                                        let mut state = state.write().unwrap();
                                        // Ensures we have enough copies of uno for all of our
                                        // friends to play
                                        state.deck = Deck::new(num_players);
                                        for _ in 0..num_players {
                                            // Draw 7 cards per player

                                            let mut hand = [Card::Wild(Color::None); 7];
                                            for i in 0..7 {
                                                hand[i] = state.deck.draw();
                                            }

                                            hands.push(hand)
                                        }
                                    }

                                    for player in state.read().unwrap().players.values() {
                                        let new_message = DynMessage::new_msg(None, Action::Started(hands.pop().unwrap()));
                                        player.send_msg(&new_message).expect("Failed to send message");
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

                                    let mut state = state.write().unwrap();
                                    let card = state.deck.draw();

                                    state.send_msg(&player_id, &DynMessage::draw(card)).expect("Draw failed");
                                },
                                Action::PlayCard(card) => {
                                    // Check if card can be played on top of current deck, if so do
                                    // it and return a success. If not then return a failure
                                    if !state.read().unwrap().in_game || state.read().unwrap().turn != player_id {
                                        continue;
                                    }
                                    let mut state = state.write().unwrap();
                                    let mut curr_turn = None;
                                    let response = match &state.deck.play(card) {
                                        Some(_) => {
                                            state.turn = *state.after(&player_id).expect("Next player invalid");
                                            curr_turn = Some(state.turn);
                                            Action::AcceptPlayCard
                                        },
                                        None => Action::DenyPlayCard
                                    };

                                    let message = DynMessage::new_msg(None, response);
                                    state.send_msg(&player_id, &message).expect("Send message fail");
                                    let new_card = DynMessage::new_msg(player_name.clone(), Action::TopCard(state.deck.get_facing().unwrap().clone()));
                                    state.broadcast(new_card).expect("Broadcast fail");
                                    if let Some(new_turn) = curr_turn {
                                        state.send_msg(&new_turn, &DynMessage::new_msg(None, Action::YourTurn)).expect("New turn failed");
                                    }
                                },
                                Action::SetName(name) => {
                                    println!("Set name: {}", name);
                                    // Set user's name to `name`
                                    state.write().unwrap().players.get_mut(&player_id).unwrap().set_name(&name);
                                    player_name = Some(name);
                                },
                                Action::DrawnCard(_) => { unreachable!("User will never initialize a DrawnCard action") },
                                Action::TopCard(_) => { unreachable!("User will never call TopCard") },
                                Action::Started(_) => { unreachable!("User will never call Started") },
                                Action::AcceptPlayCard | Action::DenyPlayCard => { unreachable!("User will never accept or deny played card") },
                                Action::YourTurn => { unreachable!("Player will never YourTurn the server >:(") },

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
}

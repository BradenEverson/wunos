use std::sync::Arc;

use server::{game::card::Card, state::msg::{Action, DynMessage}};
use test_client::hand::Hand;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{lock::Mutex, SinkExt, StreamExt};

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:7878";

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    println!("Connected to the server");

    let (mut write, mut read) = ws_stream.split();

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    println!("Enter your username: ");
    let mut username = String::new();
    reader.read_line(&mut username).await.expect("Failed to read line");
    let username = username.trim().to_string();
    let mut username_sent = false;

    let hand: Arc<Mutex<Hand>> = Arc::new(Mutex::new(Hand::default()));

    let send_messages = async {
        loop {

            let action_msg = if !username_sent {
                let sent_action = Action::SetName(username.clone());
                username_sent = true;

                serde_json::to_string(&sent_action)
            } else {
                let mut input = String::new();
                reader.read_line(&mut input).await.expect("Failed to read line");

                let action = if input.trim() == "START" {
                    Action::Start
                } else if input.trim().starts_with("play") {
                    let choice = &input.trim()[4..=5];
                    println!("{}", choice);
                    let card_choice: Result<usize, _> = choice.trim().parse();

                    if let Ok(i) = card_choice {
                        let card = {
                            let mut hand = hand.lock().await;
                            hand.last_card_choice = Some(i);
                            hand.cards[i].clone()
                        };
                        println!("Playing {}", card);
                        Action::PlayCard(card)
                    } else {
                        continue;
                    }

                } else {
                    Action::Message(input.trim().to_string())
                };

                serde_json::to_string(&action)
            };
            if let Ok(msg) = action_msg {
                let msg = Message::Text(msg.trim().to_string());
                write.send(msg).await.expect("Failed to send message");
            }
        }
    };

    let receive_messages = async {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let deserialized: Result<DynMessage, _> = serde_json::from_str(&text);

                    if let Ok(message) = deserialized {
                        let begin_msg = match message.sender {
                            Some(name) => format!("{}: ", name),
                            None => String::new()
                        };
                        match message.action {
                            Action::Message(msg) => println!("{}{}", begin_msg, msg),
                            Action::DrawnCard(card) => {
                                println!("You draw a {}", card);
                                hand.lock().await.cards.push(card)
                            },
                            Action::TopCard(card) => println!("Top Card is a {}", card),
                            Action::PlayCard(card) => println!("{} played {}", begin_msg, card),
                            Action::Started(starting_cards) => {
                                hand.lock().await.cards.extend(starting_cards.iter());
                                println!("Your hand: {:?}", hand.lock().await.cards);
                            }
                            _ => {},
                        };
                    } 
                }
                Ok(Message::Close(_)) => {
                    println!("Connection closed");
                    break;
                }
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    };

    tokio::select! {
        _ = send_messages => {},
        _ = receive_messages => {},
    }
}

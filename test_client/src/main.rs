use std::sync::{Arc, RwLock};

use server::{game::card::{Card, Color}, state::msg::{Action, DynMessage}};
use test_client::hand::Hand;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{lock::Mutex, SinkExt, StreamExt};

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:8080";

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    println!("Connected to the server");

    let (mut write, mut read) = ws_stream.split();

    let mut write = Arc::new(Mutex::new(write));

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
                    let card_choice: Result<usize, _> = choice.trim().parse();

                    if let Ok(i) = card_choice {
                        let mut card = {
                            let mut hand = hand.lock().await;
                            hand.last_card_choice = Some(i);
                            hand.cards[i].clone()
                        };
                        println!("Playing {}", card);
                        while card.color() == Color::None {
                            println!("What color would you like?");
                            let mut color_choice = String::new();
                            reader.read_line(&mut color_choice).await.expect("Read color choice");

                            let color = match color_choice.trim() {
                                "red" => Color::Red,
                                "blue" => Color::Blue,
                                "yellow" => Color::Yellow,
                                "green" => Color::Green,
                                _ => Color::None
                            };
                            card = match card {
                                Card::DrawFour(_) => Card::DrawFour(color),
                                Card::Wild(_) => Card::Wild(color),
                                _ => unreachable!("Non wild or draw four card MUST have a color")
                            };
                        }
                        Action::PlayCard(card)
                    } else {
                        continue;
                    }

                } else if input.trim().starts_with("draw") {
                    Action::DrawCard
                }else {
                    Action::Message(input.trim().to_string())
                };

                serde_json::to_string(&action)
            };
            if let Ok(msg) = action_msg {
                let msg = Message::Text(msg.trim().to_string());
                write.lock().await.send(msg).await.expect("Failed to send message");
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
                            
                            Action::TopCard(card) => println!("Top Card is a {}", card),
                            Action::PlayCard(card) => println!("{} played {}", begin_msg, card),
                            Action::DrawnCard(card) => {
                                println!("You draw a {}", card);
                                hand.lock().await.cards.push(card);
                                println!("Your hand: {:?}", hand.lock().await.cards);
                            },
                            Action::Started(starting_cards) => {
                                hand.lock().await.cards.extend(starting_cards.iter());
                                println!("Your hand: {:?}", hand.lock().await.cards);
                            },
                            Action::AcceptPlayCard => {
                                let mut hand = hand.lock().await;
                                if let Some(choice) = hand.last_card_choice {
                                    hand.cards.remove(choice);
                                    hand.last_card_choice = None;
                                    if hand.cards.len() == 0 {
                                        let win = serde_json::to_string(&Action::Win).unwrap();
                                        write.lock().await.send(Message::Text(win)).await.expect("Send win fail");
                                    }
                                } else {
                                    panic!("Card was accepted but no log of last card chosen, something is very wrong")
                                }
                            },
                            Action::DenyPlayCard => {
                                println!("That's not a valid card to choose! Please pick again >:(");
                                hand.lock().await.last_card_choice = None;
                            },
                            Action::YourTurn => {
                                println!("It's your turn!");
                                println!("Your hand: {:?}", hand.lock().await.cards);
                            },
                            Action::Skipped => {
                                println!("You've been skipped buddy");
                            },
                            Action::DrawFour(cards) => {
                                println!("You got hit with a draw 4 :(");
                                for card in cards {
                                    println!("\t+ {}", card);
                                }
                                hand.lock().await.cards.extend(cards.iter());
                                println!("Your hand: {:?}", hand.lock().await.cards);
                            },
                            Action::DrawTwo(cards) => {
                                println!("You got hit with a draw 2 :(");
                                for card in cards {
                                    println!("\t+ {}", card);
                                }
                                hand.lock().await.cards.extend(cards.iter());
                                println!("Your hand: {:?}", hand.lock().await.cards);
                            },
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

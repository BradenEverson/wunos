use server::{game::card::Card, state::msg::{Action, DynMessage}};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};

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

    let mut cards: Vec<Card> = vec![];


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
                                cards.push(card)
                            },
                            Action::TopCard(card) => println!("Top Card is a {}", card),
                            Action::PlayCard(card) => println!("{} played {}", begin_msg, card),
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

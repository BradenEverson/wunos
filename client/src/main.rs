use client::hand::Hand;
use futures::{lock::Mutex, SinkExt, StreamExt};
use ratatui::{
    backend::CrosstermBackend, layout::{Constraint, Direction, Layout, Rect}, style::{Style, Color}, widgets::{Block, Borders, List, ListItem, Paragraph}, Terminal
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use server::{game::card::Card, state::msg::{Action, DynMessage}};
use server::game::card::Color as CardColor;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use std::{collections::VecDeque, io, sync::{Arc, RwLock}};
use tokio::sync::watch;

#[derive(Copy, Clone)]
enum Screen {
    Input,
    Action,
    InGame
}

struct AppState {
    pub screen: Screen,
    pub input: String,
    pub chat_input: String,
    pub messages: VecDeque<String>,
    pub hand: Hand,
    pub top_card: Card,
    pub selected: usize
}

impl AppState {
    fn new() -> Self {
        AppState {
            screen: Screen::Input,
            input: String::new(),
            chat_input: String::new(),
            messages: VecDeque::new(),
            hand: Hand::default(),
            top_card: Card::Wild(CardColor::Red),
            selected: 0
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {

    let url = "ws://127.0.0.1:8080";

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    let (write, mut read) = ws_stream.split();

    let write = Arc::new(Mutex::new(write));


    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app_state = Arc::new(RwLock::new(AppState::new()));
    let (tx, mut rx) = watch::channel(());

    let app_state_clone = app_state.clone();
    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let deserialized: Result<DynMessage, _> = serde_json::from_str(&text);

                    if let Ok(msg) = deserialized {
                        let begin_msg = match msg.sender {
                            Some(name) => format!("{}: ", name),
                            None => String::new()
                        };

                        match msg.action {
                            Action::Message(msg) => {
                                let mut app_state = app_state_clone.write().unwrap();
                                app_state.messages.push_back(format!("{}{}", begin_msg, msg));
                                tx.send(()).unwrap();
                            }, 
                            Action::Started(starting_cards) => {
                                let mut app_state = app_state_clone.write().unwrap();
                                app_state.hand.cards.extend(starting_cards.iter());
                                app_state.screen = Screen::InGame;
                            },
                            Action::TopCard(card) => {
                                let mut app_state = app_state_clone.write().unwrap();
                                app_state.top_card = card;

                                tx.send(()).unwrap();
                            },
                            Action::DrawnCard(card) => {
                                let mut app_state = app_state_clone.write().unwrap();
                                app_state.hand.cards.push(card);

                                tx.send(()).unwrap();
                            },
                            Action::AcceptPlayCard => {
                                let mut app_state = app_state_clone.write().unwrap();
                                if let Some(choice) = app_state.hand.last_choice {
                                    app_state.hand.cards.remove(choice);
                                    app_state.hand.last_choice = None;

                                    // TODO: Win the game if hand is empty
                                }
                                tx.send(()).unwrap();
                            },
                            Action::PlayCard(card) => {
                                let mut app_state = app_state_clone.write().unwrap();
                                app_state.messages.push_back(format!("{} played {}", begin_msg, card));
                                tx.send(()).unwrap();
                            },
                            Action::DenyPlayCard => {
                                let mut app_state = app_state_clone.write().unwrap();
                                app_state.hand.last_choice = None;
                                tx.send(()).unwrap();
                            },
                            Action::YourTurn => {
                                 let mut app_state = app_state_clone.write().unwrap();
                                 app_state.messages.push_back(format!("It's your turn!"));
                                tx.send(()).unwrap();
                            },
                            Action::Skipped => {
                                 let mut app_state = app_state_clone.write().unwrap();
                                 app_state.messages.push_back(format!("YOU GOT SKIPPED BOY"));
                                 tx.send(()).unwrap();
                            },
                            Action::DrawFour(cards) => {
                                let mut app_state = app_state_clone.write().unwrap();
                                app_state.hand.cards.extend(cards.iter());
                                tx.send(()).unwrap();
                            },
                            Action::DrawTwo(cards) => {
                                let mut app_state = app_state_clone.write().unwrap();
                                app_state.hand.cards.extend(cards.iter());
                                tx.send(()).unwrap();
                            },
                            _ => {}
                        }
                    }
                }
                _ => { break }
            }
        }
    });

    loop {
        terminal.draw(|f| {
            match app_state.read().unwrap().screen {
                Screen::Input => draw_input_screen(f, app_state.clone()),
                Screen::Action => draw_action_screen(f, app_state.clone()),
                Screen::InGame => draw_game_screen(f, app_state.clone())
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let app_state = app_state.clone();

                let screen = {
                    app_state.read().unwrap().screen
                };

                match screen {
                    Screen::Input => match key.code {
                        KeyCode::Esc => {
                            break;
                        },
                        KeyCode::Char(c) => {
                            let mut app_state = app_state.write().unwrap();
                            app_state.input.push(c)
                        },
                        KeyCode::Backspace => { 
                            let mut app_state = app_state.write().unwrap();
                            app_state.input.pop(); 
                        },
                        KeyCode::Enter => {
                            // Move to the next screen on Enter
                            let mut app_state = app_state.write().unwrap();
                            app_state.screen = Screen::Action;

                            let name = Action::SetName(app_state.input.clone());
                            let action_msg = serde_json::to_string(&name).unwrap();

                            let msg = Message::text(action_msg.trim());
                            write.lock().await.send(msg).await.expect("Failed to set name");
                        },
                        _ => {}
                    },
                    Screen::Action => match key.code {
                        KeyCode::Esc => {
                            break;
                        },
                        KeyCode::Char(c) => {
                            let mut app_state = app_state.write().unwrap();
                            app_state.chat_input.push(c)
                        },
                        KeyCode::Backspace => { 
                            let mut app_state = app_state.write().unwrap();
                            app_state.chat_input.pop(); 
                        },
                        KeyCode::Enter => {
                            let mut app_state = app_state.write().unwrap();
                            let msg = if &app_state.chat_input.trim() == &"START" {
                                let string = serde_json::to_string(&Action::Start).unwrap();
                                Message::text(string)
                            } else {

                                let message = format!("{}: {}", app_state.input, app_state.chat_input);
                                app_state.messages.push_back(message.clone());

                                let action = Action::Message(app_state.chat_input.clone());
                                let action_msg = serde_json::to_string(&action).unwrap();
                                Message::text(action_msg.trim())
                            };

                            app_state.chat_input.clear();
                            write.lock().await.send(msg).await.expect("Failed to send message");
                        },
                        _ => {}
                    },
                    Screen::InGame => { 
                        let action: Option<Action> = match key.code {
                            KeyCode::Esc => break,
                            KeyCode::Left => {
                                let mut app_state = app_state.write().unwrap();
                                if app_state.selected > 0 {
                                    app_state.selected -= 1;
                                }
                                None
                            },
                            KeyCode::Right => {
                                let mut app_state = app_state.write().unwrap();
                                if app_state.selected < app_state.hand.cards.len() - 1 {
                                    app_state.selected += 1;
                                }
                                None
                            },
                            KeyCode::Char('d') => {
                                // Draw a card
                                Some(Action::DrawCard)
                            },
                            KeyCode::Enter => {
                                // Play selected
                                let mut app_state = app_state.write().unwrap();
                                let chosen_card = app_state.hand.cards[app_state.selected];
                                app_state.hand.last_choice = Some(app_state.selected);
                                Some(Action::PlayCard(chosen_card))
                            }
                            _ => None
                        };

                        if let Some(msg) = action {
                            let serialized = serde_json::to_string(&msg);
                            if let Ok(send_ser) = serialized {
                                let message = Message::Text(send_ser.trim().to_string());
                                write.lock().await.send(message).await.expect("Failed to send message");
                            }
                        }

                    }
                }
            }
        }

        if rx.has_changed().unwrap() {
            rx.borrow_and_update();
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn draw_input_screen(f: &mut ratatui::Frame, app_state: Arc<RwLock<AppState>>) {
    let size = f.size();

    let app_state = app_state.read().unwrap();
    let input = Paragraph::new(app_state.input.clone())
        .block(Block::default().borders(Borders::ALL).title("Enter your name:"));
    f.render_widget(input, size);
}

fn draw_action_screen(f: &mut ratatui::Frame, app_state: Arc<RwLock<AppState>>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Min(5),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());


    let app_state = app_state.read().unwrap();
    
    let messages: Vec<ListItem> = app_state
        .messages
        .iter()
        .map(|m| ListItem::new(m.as_str()))
        .collect();

    let messages_widget = List::new(messages)
        .block(Block::default().borders(Borders::ALL).title("Chat"));

    f.render_widget(messages_widget, chunks[0]);

    let input = Paragraph::new(app_state.chat_input.clone())
        .block(Block::default().borders(Borders::ALL).title("Message"));

    f.render_widget(input, chunks[1]);
}

fn draw_game_screen(f: &mut ratatui::Frame, app_state: Arc<RwLock<AppState>>) {
    let size = f.size();

    let block = Block::default()
        .title("UNO Game")
        .borders(Borders::ALL);
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(size);

    
    let app_state = app_state.read().unwrap();

    let top = app_state.top_card;
    let top_card_text = top.to_string();
    let top_card_paragraph = Paragraph::new(top_card_text)
        .style(Style::default().fg(color_to_tui_color(top.color())))
        .block(Block::default().title("Top Card").borders(Borders::ALL));
    f.render_widget(top_card_paragraph, chunks[0]);

    let cards = &app_state.hand.cards;

    let card_width = size.width / cards.len() as u16;
    let card_height = card_width * 2;
    let mut constraints: Vec<Constraint> = vec![];
    for _ in 0..cards.len() {
        constraints.push(Constraint::Length(card_width));
    }
    let card_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints.clone())
        .split(Rect::new(chunks[1].x, chunks[1].y, size.width, card_height));

    for (i, &card) in cards.iter().enumerate() {
        let style = if i == app_state.selected {
            Style::default().bg(color_to_tui_color(card.color())).fg(Color::White)
        } else {
            Style::default()
        };
        let card_text = card.to_string();
        let card_block = Paragraph::new(card_text)
            .style(style)
            .block(Block::default().borders(Borders::ALL).style(Style::default().fg(color_to_tui_color(card.color()))));
        f.render_widget(card_block, card_chunks[i]);
    }
}


fn color_to_tui_color(color: CardColor) -> Color {
    match color {
        CardColor::None => Color::White,
        CardColor::Red => Color::Red,
        CardColor::Yellow => Color::Yellow,
        CardColor::Green => Color::Green,
        CardColor::Blue => Color::Blue,
    }
}

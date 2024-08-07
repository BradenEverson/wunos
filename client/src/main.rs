use client::hand::Hand;
use futures::{lock::Mutex, SinkExt, StreamExt};
use ratatui::{
    backend::CrosstermBackend, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Style, Stylize}, widgets::{Block, Borders, List, ListItem, Paragraph}, Terminal
};
use crossterm::{
    event::{self, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode}
};
use server::{game::card::Card, state::msg::{Action, DynMessage}};
use server::game::card::Color as CardColor;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use std::{collections::VecDeque, fmt::write, io, sync::{Arc, RwLock}};

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
    pub selected: usize,
    pub wild_color: CardColor,
    pub won: bool
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
            selected: 0,
            wild_color: CardColor::None,
            won: false
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
                                {
                                    let app_state = app_state_clone.clone();
                                    let mut app_state = app_state.write().unwrap();

                                    app_state.messages.push_back(format!("{}{}", begin_msg, msg));
                                }
                            }, 
                            Action::Started(starting_cards) => {
                                {
                                    let app_state = app_state_clone.clone();
                                    let mut app_state = app_state.write().unwrap();

                                    app_state.hand.cards.extend(starting_cards.iter());
                                    app_state.screen = Screen::InGame;
                                }
                            },
                            Action::TopCard(card) => {
                                {
                                    let app_state = app_state_clone.clone();
                                    let mut app_state = app_state.write().unwrap();

                                    app_state.top_card = card;
                                }
                            },
                            Action::DrawnCard(card) => {
                                {
                                    let app_state = app_state_clone.clone();
                                    let mut app_state = app_state.write().unwrap();

                                    app_state.hand.cards.push(card);
                                }
                            },
                            Action::AcceptPlayCard => {
                                {
                                    let app_state = app_state_clone.clone();
                                    let mut app_state = app_state.write().unwrap();
                                    if let Some(choice) = app_state.hand.last_choice {
                                        app_state.hand.cards.remove(choice);
                                        app_state.hand.last_choice = None;

                                        if app_state.hand.cards.len() == 0 {
                                            println!("Win");
                                            app_state.won = true;
                                        }
                                    }
                                }
                            },
                            Action::PlayCard(card) => {
                                {
                                    let app_state = app_state_clone.clone();
                                    let mut app_state = app_state.write().unwrap();

                                    app_state.messages.push_back(format!("{} played {}", begin_msg, card));
                                }
                            },
                            Action::DenyPlayCard => {
                                {
                                    let app_state = app_state_clone.clone();
                                    let mut app_state = app_state.write().unwrap();

                                    app_state.hand.last_choice = None;
                                }
                            },
                            Action::YourTurn => {
                                {
                                    let app_state = app_state_clone.clone();
                                    let mut app_state = app_state.write().unwrap();

                                    app_state.messages.push_back(format!("It's your turn!"));
                                }
                            },
                            Action::Skipped => {
                                {
                                    let app_state = app_state_clone.clone();
                                    let mut app_state = app_state.write().unwrap();

                                    app_state.messages.push_back(format!("YOU GOT SKIPPED BOY"));
                                }
                            },
                            Action::DrawFour(cards) => {
                                {
                                    let app_state = app_state_clone.clone();
                                    let mut app_state = app_state.write().unwrap();

                                    app_state.hand.cards.extend(cards.iter());
                                }
                            },
                            Action::DrawTwo(cards) => {
                                {
                                    let app_state = app_state_clone.clone();
                                    let mut app_state = app_state.write().unwrap();

                                    app_state.hand.cards.extend(cards.iter());
                                }
                            },
                            Action::Win => {
                                {
                                    let app_state = app_state_clone.clone();
                                    let mut app_state = app_state.write().unwrap();

                                    app_state.won = false;
                                    app_state.hand.cards = vec![];

                                    app_state.screen = Screen::Action;
                                }
                            }
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
            let screen = {
                app_state.read().unwrap().screen
            };

            match screen {
                Screen::Input => draw_input_screen(f, app_state.clone()),
                Screen::Action => draw_action_screen(f, app_state.clone()),
                Screen::InGame => draw_game_screen(f, app_state.clone())
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let app_state = app_state.clone();

                {
                    let mut app_state = app_state.write().unwrap();
                    app_state.selected = app_state.selected.clamp(0, app_state.hand.cards.len());
                }

                let screen = {
                    app_state.read().unwrap().screen
                };

                let won = { 
                    app_state.read().unwrap().won
                };

                if won {
                    {
                        let msg = Message::text(serde_json::to_string(&Action::Win).unwrap());
                        write.lock().await.send(msg).await.expect("Failed to send win");
                    }
                }

                match screen {
                    Screen::Input => match key.code {
                        KeyCode::Esc => {
                            break;
                        },
                        KeyCode::Char(c) => {
                            {
                                let app_state = app_state.clone();
                                let mut app_state = app_state.write().unwrap();
                                app_state.input.push(c)
                            }
                        },
                        KeyCode::Backspace => { 
                            let app_state = app_state.clone();
                            let mut app_state = app_state.write().unwrap();
                            app_state.input.pop(); 
                        },
                        KeyCode::Enter => {
                            // Move to the next screen on Enter
                            let app_state = app_state.clone();
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
                            let app_state = app_state.clone();
                            let mut app_state = app_state.write().unwrap();
                            app_state.chat_input.push(c)
                        },
                        KeyCode::Backspace => { 
                            let app_state = app_state.clone();
                            let mut app_state = app_state.write().unwrap();
                            app_state.chat_input.pop(); 
                        },
                        KeyCode::Enter => {
                            let app_state = app_state.clone();
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
                            KeyCode::Left | KeyCode::Char('h') => {
                                let app_state = app_state.clone();
                                let mut app_state = app_state.write().unwrap();
                                if app_state.selected == 0 {
                                    app_state.selected = app_state.hand.cards.len() - 1;
                                } else {
                                    app_state.selected -= 1;
                                }
                                None
                            },
                            KeyCode::Right | KeyCode::Char('l') => {
                                let app_state = app_state.clone();
                                let mut app_state = app_state.write().unwrap();
                                if app_state.selected == app_state.hand.cards.len() - 1 {
                                    app_state.selected = 0;
                                } else {
                                    app_state.selected += 1;
                                }
                                None
                            },
                            KeyCode::Char('d') | KeyCode::Char(' ') => {
                                Some(Action::DrawCard)
                            },
                            KeyCode::Char('r') | KeyCode::Char('1') => {
                                let app_state = app_state.clone();
                                let mut app_state = app_state.write().unwrap();
            
                                app_state.wild_color = CardColor::Red;

                                None
                            },
                            KeyCode::Char('y') | KeyCode::Char('2') => {
                                let app_state = app_state.clone();
                                let mut app_state = app_state.write().unwrap();
            
                                app_state.wild_color = CardColor::Yellow;

                                None
                            },
                            KeyCode::Char('b') | KeyCode::Char('3') => {
                                let app_state = app_state.clone();
                                let mut app_state = app_state.write().unwrap();

                                app_state.wild_color = CardColor::Blue;

                                None
                            },
                            KeyCode::Char('g') | KeyCode::Char('4') => {
                                let app_state = app_state.clone();
                                let mut app_state = app_state.write().unwrap();
            
                                app_state.wild_color = CardColor::Green;

                                None
                            },
                            KeyCode::Enter => {
                                let app_state = app_state.clone();
                                let mut app_state = app_state.write().unwrap();
                                let chosen_card = app_state.hand.cards[app_state.selected];
                                if chosen_card.color() == CardColor::None {
                                    if app_state.wild_color != CardColor::None {
                                        let card = match chosen_card {
                                            Card::DrawFour(_) => Card::DrawFour(app_state.wild_color),
                                            Card::Wild(_) => Card::Wild(app_state.wild_color),
                                            _ => unreachable!("No card other than +4 and Wild will ever have None as a color")
                                        };
                                        app_state.hand.last_choice = Some(app_state.selected);
                                        Some(Action::PlayCard(card))
                                    } else {
                                        None
                                    }
                                } else {
                                    app_state.hand.last_choice = Some(app_state.selected);
                                    Some(Action::PlayCard(chosen_card))
                                }
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

    let app_state = app_state.read().unwrap();
    let chosen_color_none = 
        app_state.hand.cards.len() > app_state.selected 
        && app_state.hand.cards[app_state.selected].color() == CardColor::None;

    let size = f.size();

    let block = Block::default()
        .title("UNO Game")
        .borders(Borders::ALL);
    f.render_widget(block, size);

    let chunks = match chosen_color_none {
        true => {
            Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                    Constraint::Max(3),
                    Constraint::Length(6),
                    Constraint::Min(0),
                    Constraint::Max(4)
                    ]
                    .as_ref(),
                )
                .split(size)
        },
        false => {
            Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                    Constraint::Max(3),
                    Constraint::Length(6),
                    Constraint::Min(0),
                    ]
                    .as_ref(),
                )
                .split(size)
        }
    };
     
    let message = app_state
        .messages
        .iter()
        .map(|m| ListItem::new(m.as_str()))
        .last()
        .unwrap_or(ListItem::new(""));

    let messages_widget = List::new(vec![message])
        .block(Block::default().borders(Borders::ALL).fg(Color::DarkGray));

    f.render_widget(messages_widget, chunks[0]);

    let top = app_state.top_card;
    let top_card_text = top.to_string();
    let top_card_paragraph = Paragraph::new(top_card_text)
        .style(Style::default()
            .fg(Color::White)
            .bg(color_to_tui_color(top.color())))
        .block(Block::default().title("Top Card")
            .borders(Borders::ALL));

    f.render_widget(top_card_paragraph, chunks[1]);

    let cards = &app_state.hand.cards;

    let card_width = if cards.len() != 0 {
        size.width / cards.len() as u16
    } else {
        size.width
    };

    let card_height = card_width * 2;
    let mut constraints: Vec<Constraint> = vec![];
    for _ in 0..cards.len() {
        constraints.push(Constraint::Length(card_width));
    }
    let card_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints.clone())
        .split(Rect::new(chunks[2].x, chunks[2].y, size.width, card_height));

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

    if chosen_color_none {

        let colors = [CardColor::Red, CardColor::Yellow, CardColor::Blue, CardColor::Green];

        let card_width = if colors.len() != 0 {
            size.width / colors.len() as u16
        } else {
            size.width
        };

        let card_height = card_width * 2;
        let mut constraints: Vec<Constraint> = vec![];
        for _ in 0..colors.len() {
            constraints.push(Constraint::Length(card_width));
        }

        let card_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints.clone())
            .split(Rect::new(chunks[3].x, chunks[3].y, size.width, card_height));

        for (i, &color) in colors.iter().enumerate() {
            let style = if color == app_state.wild_color {
                Style::default().bg(color_to_tui_color(color)).fg(Color::White)
            } else {
                Style::default()
            };
            let card_text = color.to_string();
            let card_block = Paragraph::new(card_text)
                .style(style)
                .block(Block::default().borders(Borders::ALL).style(Style::default().fg(color_to_tui_color(color))));
            f.render_widget(card_block, card_chunks[i]);
        }       
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

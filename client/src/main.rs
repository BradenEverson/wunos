use futures::{lock::Mutex, SinkExt, StreamExt};
use ratatui::{
    backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, widgets::{Block, Borders, List, ListItem, Paragraph}, Terminal
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use server::state::msg::{Action, DynMessage};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use std::{collections::VecDeque, io, sync::{Arc, RwLock}};
use tokio::sync::watch;

enum Screen {
    Input,
    Action,
    InGame
}

struct AppState {
    screen: Screen,
    input: String,
    chat_input: String,
    messages: VecDeque<String>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            screen: Screen::Input,
            input: String::new(),
            chat_input: String::new(),
            messages: VecDeque::new(),
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
                _ => todo!()
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let mut app_state = app_state.write().unwrap();
                match app_state.screen {
                    Screen::Input => match key.code {
                        KeyCode::Esc => {
                            break;
                        },
                        KeyCode::Char(c) => app_state.input.push(c),
                        KeyCode::Backspace => { app_state.input.pop(); },
                        KeyCode::Enter => {
                            // Move to the next screen on Enter
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
                        KeyCode::Char(c) => app_state.chat_input.push(c),
                        KeyCode::Backspace => { app_state.chat_input.pop(); },
                        KeyCode::Enter => {
                            let message = format!("{}: {}", app_state.input, app_state.chat_input);
                            app_state.messages.push_back(message.clone());

                            let action = Action::Message(app_state.chat_input.clone());
                            let action_msg = serde_json::to_string(&action).unwrap();
                            let msg = Message::text(action_msg.trim());

                            app_state.chat_input.clear();
                            write.lock().await.send(msg).await.expect("Failed to send message");
                        },
                        _ => {}
                    },
                    _ => todo!()
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

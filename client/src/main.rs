use ratatui::{
    backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, widgets::{Block, Borders, List, ListItem, Paragraph}, Terminal
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::{collections::VecDeque, io};

enum Screen {
    Input,
    Action,
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

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app_state = AppState::new();

    loop {
        terminal.draw(|f| {
            match app_state.screen {
                Screen::Input => draw_input_screen(f, &app_state),
                Screen::Action => draw_action_screen(f, &app_state),
            }
        })?;

        if let Event::Key(key) = event::read()? {
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
                        app_state.messages.push_back(message);
                        app_state.chat_input.clear();
                    },
                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn draw_input_screen(f: &mut ratatui::Frame, app_state: &AppState) {
    let size = f.size();

    let input = Paragraph::new(app_state.input.clone())
        .block(Block::default().borders(Borders::ALL).title("Enter your name:"));
    f.render_widget(input, size);
}

fn draw_action_screen(f: &mut ratatui::Frame, app_state: &AppState) {
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

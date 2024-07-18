use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

struct Player {
    name: String,
    card_count: usize,
}

struct UnoGame {
    players: Vec<Player>,
    current_card: String,
    your_hand: Vec<String>,
    turn: usize,
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let game = UnoGame {
        players: vec![
            Player { name: String::from("Alice"), card_count: 5 },
            Player { name: String::from("Bob"), card_count: 3 },
            Player { name: String::from("Charlie"), card_count: 7 },
        ],
        current_card: String::from("Red 5"),
        your_hand: vec![String::from("Blue 7"), String::from("Green Skip"), String::from("Yellow 2")],
        turn: 0,
    };

    let mut selected_card = 0;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(60),
                    ]
                    .as_ref(),
                )
                .split(size);

            let players: Vec<ListItem> = game
                .players
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let player_style = if i == game.turn {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    ListItem::new(Text::styled(format!("{}: {} cards", p.name, p.card_count), player_style))
                })
                .collect();

            let players_list = List::new(players)
                .block(Block::default().borders(Borders::ALL).title("Players"));

            f.render_widget(players_list, chunks[0]);

            let current_card = Paragraph::new(Text::styled(format!("Current Card: {}", game.current_card), get_card_style(&game.current_card)))
                .block(Block::default().borders(Borders::ALL).title("Current Card"));

            f.render_widget(current_card, chunks[1]);

            let your_hand: Vec<ListItem> = game
                .your_hand
                .iter()
                .enumerate()
                .map(|(i, card)| {
                    let card_style = if i == selected_card {
                        get_card_style(card).add_modifier(Modifier::REVERSED)
                    } else {
                        get_card_style(card)
                    };
                    ListItem::new(Text::styled(card, card_style))
                })
                .collect();

            let hand_list = List::new(your_hand)
                .block(Block::default().borders(Borders::ALL).title("Your Hand"));

            f.render_widget(hand_list, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected_card < game.your_hand.len() - 1 {
                        selected_card += 1;
                    }
                },
                KeyCode::Up | KeyCode::Char('k') => {
                    if selected_card > 0 {
                        selected_card -= 1;
                    }
                },
                KeyCode::Enter => {
                    if game.turn == 0 {
                        // Send a card over for validation
                    }
                },
                KeyCode::Char('d') => {
                    if game.turn == 0 {
                        // Draw a card
                    }
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn get_card_style(card: &str) -> Style {
    if card.contains("Red") {
        Style::default().fg(Color::Red)
    } else if card.contains("Blue") {
        Style::default().fg(Color::Blue)
    } else if card.contains("Green") {
        Style::default().fg(Color::Green)
    } else if card.contains("Yellow") {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    }
}

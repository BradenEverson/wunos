use ratatui::{
    backend::CrosstermBackend, layout::{Constraint, Direction, Layout, Rect}, style::{Color as TuiColor, Style}, widgets::{Block, Borders, List, ListItem, Paragraph, Widget}, Terminal
};

use server::game::card::Color as CardColor;
use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode}, style::Color};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use server::game::card::Card;
use std::io::{self, stdout};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let cards = vec![
        Card::Normal(CardColor::Red, 5),
        Card::DrawTwo(CardColor::Green),
        Card::Reverse(CardColor::Blue),
        Card::Skip(CardColor::Yellow),
        Card::Wild(CardColor::None),
        Card::DrawFour(CardColor::None),
    ];

    let top_card = Card::Normal(CardColor::Blue, 3);
    let mut selected = 0;

    loop {
        terminal.draw(|f| draw_game_screen(f, &cards, top_card, selected))?;

        if crossterm::event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Left => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    },
                    KeyCode::Right => {
                        if selected < cards.len() - 1 {
                            selected += 1;
                        }
                    }, 
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn draw_game_screen(f: &mut ratatui::Frame, cards: &[Card], top: Card, selected_index: usize) {
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

    // Display top card
    let top_card_text = top.to_string();
    let top_card_paragraph = Paragraph::new(top_card_text)
        .style(Style::default().fg(color_to_tui_color(top.color())))
        .block(Block::default().title("Top Card").borders(Borders::ALL));
    f.render_widget(top_card_paragraph, chunks[0]);

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
        let style = if i == selected_index {
            Style::default().bg(TuiColor::LightGreen).fg(TuiColor::Black)
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


fn color_to_tui_color(color: CardColor) -> TuiColor {
    match color {
        CardColor::None => TuiColor::Reset,
        CardColor::Red => TuiColor::Red,
        CardColor::Yellow => TuiColor::Yellow,
        CardColor::Green => TuiColor::Green,
        CardColor::Blue => TuiColor::Blue,
    }
}

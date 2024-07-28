use std::{fmt::Display, slice::Iter};

use serde::{Deserialize, Serialize};

const SKIP: u8 = 10;
const REVERSE: u8 = 11;
const PLUS_TWO: u8 = 12;
const PLUS_FOUR: u8 = 13;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Card {
    Normal(Color, u8),
    DrawTwo(Color),
    Reverse(Color),
    Skip(Color),

    Wild(Color),
    DrawFour(Color),
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Card::Normal(color, number) => format!("{} {}", color, number),
            Card::DrawTwo(color) => format!("{} Plus Two", color),
            Card::DrawFour(color) => {
                if color == &Color::None {
                    format!("Plus Four")
                } else {
                    format!("{} Plus Four", color)
                }
            },
            Card::Reverse(color) => format!("{} Reverse", color),
            Card::Skip(color) => format!("{} Skip", color),
            Card::Wild(color) => {
                if color == &Color::None {
                    format!("Wild")
                } else {
                    format!("{} Wild", color)
                }
            }
        })
    }
}

impl Card {
    pub fn color(&self) -> Color {
        match self {
            Card::Normal(color, _) 
                | Card::DrawTwo(color) 
                | Card::Skip(color) 
                | Card::Wild(color) 
                | Card::DrawFour(color) 
                | Card::Reverse(color) => {
                *color
            }
        }
    }

    pub fn number(&self) -> u8 {
        match self {
            Card::Normal(_, num) => *num,
            Card::Wild(_) => 99,
            Card::DrawFour(_) => PLUS_FOUR,
            Card::Reverse(_) => REVERSE,
            Card::Skip(_) => SKIP,
            Card::DrawTwo(_) => PLUS_TWO
        }
    }
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {

        match other {
            Card::Wild(_) | Card::DrawFour(_) => { return true },
            _ => {}
        };

        if self.color() == other.color() {
            return true;
        }
        
        if self.number() == other.number() {
            return true;
        }

        false
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Color {
    None,
    Red,
    Yellow,
    Green,
    Blue
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Color::Red => "Red",
            Color::Blue => "Blue",
            Color::Yellow => "Yellow",
            Color::Green => "Green",
            Color::None => "Wild"
        })
    }
}

impl Color {
    pub fn iterator() -> Iter<'static, Color> {
        static COLORS: [Color; 4] = [Color::Red, Color::Yellow, Color::Green, Color::Blue];

        COLORS.iter()
    }
}


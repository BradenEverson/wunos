use std::slice::Iter;

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

impl Color {
    pub fn iterator() -> Iter<'static, Color> {
        static COLORS: [Color; 4] = [Color::Red, Color::Yellow, Color::Green, Color::Blue];

        COLORS.iter()
    }
}


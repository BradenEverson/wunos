use std::slice::Iter;

#[derive(Copy, Clone, Debug)]
pub enum Card {
    Normal(Color, u8),
    DrawTwo(Color),
    Reverse(Color),
    Skip(Color),
    Wild,
    DrawFour
}

#[derive(Copy, Clone, Debug)]
pub enum Color {
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

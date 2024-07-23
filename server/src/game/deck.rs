use super::card::{Card, Color};
use rand::{seq::SliceRandom, thread_rng};


pub struct Deck {
    deck: Vec<Card>,
    facing: Vec<Card>,
    in_play: Vec<Card>
}

impl Default for Deck {
    fn default() -> Self {
        Self::new(1)
    }
}

impl Deck {
    pub fn new(times: usize) -> Self {
        let mut deck = vec![];
        let facing = vec![];
        let in_play = vec![];


        for _ in 0..times {
            for color in Color::iterator() {
                deck.push(Card::Normal(*color, 0));

                for i in 1..=9 {
                    deck.push(Card::Normal(*color, i));
                    deck.push(Card::Normal(*color, i));
                }
                deck.push(Card::DrawTwo(*color));
                deck.push(Card::DrawTwo(*color));

                deck.push(Card::Reverse(*color));
                deck.push(Card::Reverse(*color));

                deck.push(Card::Skip(*color));
                deck.push(Card::Skip(*color));
            }

            for _ in 0..4 {
                deck.push(Card::Wild(Color::None));
                deck.push(Card::Wild(Color::None));
            }
        }

        deck.shuffle(&mut thread_rng());

        Self { deck, facing, in_play }
    }

    pub fn reshuffle(&mut self) {
        self.deck.extend(self.facing.drain(0..self.facing.len()));

        self.deck.shuffle(&mut thread_rng());
    }

    pub fn deck_size(&self) -> usize {
        self.deck.len()
    }

    pub fn start_game(&mut self) {
        if let Some(card) = self.deck.pop() {
            self.facing.push(card)
        }
    }

    pub fn get_facing(&self) -> Option<&Card> {
        match self.facing.len() {
            0 => None,
            _ => Some(&self.facing[self.facing.len() - 1])
        }
    }

    #[cfg(test)]
    pub fn set_facing(&mut self, face: Card) {
        self.facing.push(face)
    }

    pub fn play(&mut self, to_play: Card) -> Option<&Card> {

        let curr = self.get_facing()?;

        if curr == &to_play {
            self.in_play.push(to_play);
            Some(&self.in_play[self.in_play.len() - 1])
        } else {
            None
        }
    }

    pub fn draw(&mut self) -> Card {
        if let Some(card) = self.deck.pop() {
            card
        } else {
            self.reshuffle();
            self.deck.pop().unwrap()
        }
    }

}

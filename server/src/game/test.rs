#[cfg(test)]
mod test {
    use crate::game::{card::{Card, Color}, deck::Deck};

    #[test]
    fn ensure_deck_construction() {
        let mut deck: Deck = Deck::new();

        assert_eq!(108, deck.deck_size());

        let _ = deck.draw();
        assert_eq!(107, deck.deck_size());
    }

    #[test]
    fn test_play_valid() {
        let mut deck = Deck::new();

        deck.set_facing(Card::Normal(Color::Red, 9));

        // Color match
        let card_option_one = Card::Normal(Color::Red, 3);
        // Number match
        let card_option_two = Card::Normal(Color::Yellow, 9);
        // Color match special cards
        let card_option_three = Card::Skip(Color::Red);
        let card_option_four = Card::Reverse(Color::Red);
        let card_option_five = Card::DrawTwo(Color::Red);

        // Wild Cards
        let card_option_six = Card::DrawFour(Color::None);
        let card_option_seven = Card::Wild(Color::None);

        assert!(deck.play(card_option_one).is_some());
        deck.set_facing(Card::Normal(Color::Red, 9));

        assert!(deck.play(card_option_two).is_some());
        deck.set_facing(Card::Normal(Color::Red, 9));

        assert!(deck.play(card_option_three).is_some());
        deck.set_facing(Card::Normal(Color::Red, 9));

        assert!(deck.play(card_option_four).is_some());
        deck.set_facing(Card::Normal(Color::Red, 9));

        assert!(deck.play(card_option_five).is_some());
        deck.set_facing(Card::Normal(Color::Red, 9));

        assert!(deck.play(card_option_six).is_some());
        deck.set_facing(Card::Normal(Color::Red, 9));

        assert!(deck.play(card_option_seven).is_some());
    }

    #[test]
    fn test_invalid_plays() {
        let mut deck = Deck::new();

        deck.set_facing(Card::Normal(Color::Red, 9));

        let invalid_one = Card::Normal(Color::Yellow, 1);

        assert!(deck.play(invalid_one).is_none());
    }
}

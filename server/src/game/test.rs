#[cfg(test)]
mod test {
    use crate::game::deck::Deck;

    #[test]
    fn ensure_deck_construction() {
        let mut deck: Deck = Deck::new();

        assert_eq!(108, deck.deck_size());
        // this will very unlikely fail, as we are trying to ensure it isn't just in order and is
        // shuffled. But the shuffle could technically shuffle exactly the same
        let _ = deck.draw();
        assert_eq!(107, deck.deck_size());
    }
}

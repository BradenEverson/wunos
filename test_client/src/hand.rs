use server::game::card::Card;

#[derive(Default)]
pub struct Hand {
    pub cards: Vec<Card>,
    pub last_card_choice: Option<usize>
}

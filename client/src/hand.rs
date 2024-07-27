use server::game::card::Card;

#[derive(Default)]
pub struct Hand {
    pub cards: Vec<Card>,
    pub last_choice: Option<usize>
}

use rand::Rng;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::res::err::Result;

#[derive(Clone, Debug)]
pub struct Player {
    id: Uuid,
    txt_color: (u8, u8, u8),
    connection: UnboundedSender<warp::ws::Message>,
    pub name: Option<String>
}

impl Player {
    pub fn new(connection: UnboundedSender<warp::ws::Message>) -> Self {
        Self { id: Uuid::new_v4(), connection, name: None, txt_color: gen_color() }
    }

    pub fn get_color(&self) -> (u8, u8, u8) {
        self.txt_color
    }

    pub fn get_name(&self) -> Option<&str> {
        match &self.name {
            Some(inner_name) => Some(inner_name),
            None => None
        }
    }

    pub fn set_name(&mut self, new_name: &str) -> Option<String> {
        let tmp_name = self.name.clone();
        self.name = Some(new_name.into());

        tmp_name
    }

    pub fn send_msg(&self, message: &warp::ws::Message) -> Result<()> {
        self.connection.send(message.clone())?;

        Ok(())
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

fn gen_color() -> (u8, u8, u8) {
    let mut rng = rand::thread_rng();

    (
        rng.gen_range(1..=255),
        rng.gen_range(1..=255),
        rng.gen_range(1..=255)
    )
}

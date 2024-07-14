use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::res::err::Result;

#[derive(Clone)]
pub struct Player {
    id: Uuid,
    connection: UnboundedSender<warp::ws::Message>,
    name: Option<String>
}

impl Player {
    pub fn new(connection: UnboundedSender<warp::ws::Message>) -> Self {
        Self { id: Uuid::new_v4(), connection, name: None }
    }

    pub fn set_name(&mut self, new_name: &str) {
        self.name = Some(new_name.into())
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

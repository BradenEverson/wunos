use tokio::sync::mpsc::UnboundedSender;

use crate::res::err::Result;

pub struct Player {
    connection: UnboundedSender<warp::ws::Message>,
    name: Option<String>
}

impl Player {
    pub fn new(connection: UnboundedSender<warp::ws::Message>) -> Self {
        Self { connection, name: None }
    }

    pub fn set_name(&mut self, new_name: &str) {
        self.name = Some(new_name.into())
    }

    pub fn send_msg(&self, message: &warp::ws::Message) -> Result<()> {
        self.connection.send(message.clone())?;

        Ok(())
    }
}

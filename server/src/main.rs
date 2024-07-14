use std::sync::Arc;

use server::{client::handler::handle_connection, res::err::Result, state::state_man::GameState};
use warp::Filter;


#[tokio::main]
async fn main() -> Result<()> {
    let state = Arc::new(GameState::new());

    let routes = warp::path::end()
        .and(warp::ws())
        .and(with_state(state.clone()))
        .map(|ws: warp::ws::Ws, state: Arc<GameState>| {
            ws.on_upgrade(move |socket| handle_connection(socket, state))
        });

    println!("Listening on 127.0.0.1:7878");
    warp::serve(routes).run(([127,0,0,1], 7878)).await;


    Ok(())
}

fn with_state(state: Arc<GameState>) -> impl Filter<Extract = (Arc<GameState>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

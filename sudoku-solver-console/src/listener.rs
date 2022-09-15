mod client;
mod handlers;
mod ws;

use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::Mutex;
use warp::{Filter, Rejection};

type Clients = Arc<Mutex<HashMap<String, client::Client>>>;
type Result<T> = std::result::Result<T, Rejection>;

pub async fn listen() {
    println!("Listening...");

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    let ws_route = warp::path::end()
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .and_then(handlers::ws_handler);

    let routes = ws_route.with(warp::cors().allow_any_origin());
    warp::serve(routes).run(([127, 0, 0, 1], 4545)).await;
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

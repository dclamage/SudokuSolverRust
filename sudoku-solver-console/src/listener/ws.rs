use super::client::Client;
use super::Clients;
use futures::{FutureExt, StreamExt};
use standard_constraints::message_handler::*;
use tokio::{runtime::Handle, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};

pub async fn client_connection(ws: WebSocket, clients: Clients) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);

    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            println!("Error sending websocket msg: {}", e);
        }
    }));

    let uuid = Uuid::new_v4().simple().to_string();

    let new_client = Client {
        client_id: uuid.clone(),
        sender: Some(client_sender),
    };

    clients.lock().await.insert(uuid.clone(), new_client);

    println!("Client {} connected", uuid);
    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                println!("Error receiving message for id {}): {}", uuid.clone(), e);
                break;
            }
        };
        client_msg(&uuid, msg, &clients).await;
    }

    clients.lock().await.remove(&uuid);
    println!("Client {} Disconnected", uuid);
}

struct SendResultWS {
    clients: Clients,
    client_id: String,
}

impl SendResultWS {
    fn new(clients: Clients, client_id: String) -> Self {
        Self { clients, client_id }
    }
}

impl SendResult for SendResultWS {
    fn send_result(&mut self, result: &str) {
        let handle = Handle::current();
        let _ = handle.enter();
        futures::executor::block_on(async {
            let locked = self.clients.lock().await;
            match locked.get(&self.client_id) {
                Some(v) => {
                    if let Some(sender) = &v.sender {
                        let _ = sender.send(Ok(Message::text(result)));
                    }
                }
                None => return,
            }
        });
    }
}

async fn client_msg(client_id: &str, msg: Message, clients: &Clients) {
    let message = match msg.to_str() {
        Ok(v) => v.to_string(),
        Err(_) => return,
    };

    let client_id = client_id.to_owned();
    let clients = clients.clone();
    let _ = tokio::task::spawn_blocking(move || {
        let send_result = Box::new(SendResultWS::new(clients, client_id));
        let mut message_handler = MessageHandler::new(send_result);
        message_handler.handle_message(&message);
    })
    .await;
}

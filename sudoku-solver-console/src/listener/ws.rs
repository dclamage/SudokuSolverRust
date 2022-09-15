use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use super::client::Client;
use super::Clients;
use futures::{FutureExt, StreamExt};
use standard_constraints::message_handler::*;
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};

pub async fn client_connection(ws: WebSocket, clients: Clients) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::channel(5);

    let client_rcv = ReceiverStream::new(client_rcv);

    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            println!("Error sending websocket msg: {}", e);
        }
    }));

    let uuid = Uuid::new_v4().simple().to_string();

    let new_client = Client {
        client_id: uuid.clone(),
        sender: None,
    };

    clients.lock().await.insert(uuid.clone(), new_client);

    println!("Client {} connected", uuid);

    let mut handler = ThreadedHandler::new(client_sender.clone()).await;

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                println!("Error receiving message for id {}): {}", uuid.clone(), e);
                break;
            }
        };

        if !handler.is_ready() {
            handler.cancel().await;
            handler = ThreadedHandler::new(client_sender.clone()).await;
        }

        if handler.send(msg).await.is_err() {
            break;
        }
    }

    handler.cancel().await;

    clients.lock().await.remove(&uuid);
    println!("Client {} Disconnected", uuid);
}

struct SendResultForWS {
    sender: Sender<Result<Message, warp::Error>>,
}

impl SendResultForWS {
    fn new(sender: Sender<Result<Message, warp::Error>>) -> Self {
        Self { sender }
    }
}

impl SendResult for SendResultForWS {
    /// Panics if we cannot send, which will terminate the thread
    fn send_result(&mut self, result: &str) {
        self.sender
            .blocking_send(Ok(Message::text(result)))
            .unwrap();
    }
}

struct ThreadedHandler {
    join_handle: std::thread::JoinHandle<()>,
    sender: Sender<Message>,
    cancel_token: Arc<AtomicBool>,
    ready_token: Arc<AtomicBool>,
}

impl ThreadedHandler {
    async fn new(client_sender: Sender<Result<Message, warp::Error>>) -> Self {
        let (handler_sender, mut handler_recv) = mpsc::channel::<Message>(5);
        let cancel_token = Arc::new(AtomicBool::from(false));
        let ready_token = Arc::new(AtomicBool::new(true));

        let joiner = std::thread::spawn({
            let cancel_token = Arc::clone(&cancel_token);
            let ready_token = Arc::clone(&ready_token);
            move || {
                // This is the thread for handling messages from the client.
                // We handle multiple messages before we give up
                let mut message_handler =
                    MessageHandler::new(Box::new(SendResultForWS::new(client_sender)));
                while let Some(message) = handler_recv.blocking_recv() {
                    let message = match message.to_str() {
                        Ok(v) => v.to_string(),
                        Err(_) => break,
                    };
                    ready_token.store(false, Ordering::SeqCst);
                    message_handler.handle_message(&message);
                    ready_token.store(true, Ordering::SeqCst);
                }
            }
        });

        ThreadedHandler {
            join_handle: joiner,
            sender: handler_sender,
            cancel_token,
            ready_token,
        }
    }

    async fn send(&self, message: Message) -> Result<(), mpsc::error::SendError<Message>> {
        self.sender.send(message).await
    }

    async fn cancel(self) {
        let Self {
            join_handle,
            sender,
            cancel_token,
            ready_token,
        } = self;
        cancel_token.store(true, Ordering::SeqCst);
        drop(sender);
        drop(ready_token);
        let mut counter = 0;
        const MAX_COUNT: usize = 100;
        while counter < MAX_COUNT && !join_handle.is_finished() {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            counter += 1;
        }
        if counter == MAX_COUNT {
            println!("Handler thread didn't stop in time, CPU may be wasted for a while");
        }
    }

    fn is_ready(&self) -> bool {
        self.ready_token.load(Ordering::SeqCst)
    }
}

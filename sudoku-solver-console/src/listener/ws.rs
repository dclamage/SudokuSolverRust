use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use super::client::Client;
use super::Clients;
use futures::{FutureExt, StreamExt};
use standard_constraints::message_handler::*;
use sudoku_solver_lib::prelude::*;
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
            println!("Error sending websocket msg: {e}");
        }
    }));

    let uuid = Uuid::new_v4().simple().to_string();

    let new_client = Client { client_id: uuid.clone(), sender: None };

    clients.lock().await.insert(uuid.clone(), new_client);

    println!("Client {uuid} connected");

    let mut handler = ThreadedHandler::new(client_sender.clone()).await;

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                println!("Error receiving message for id {}): {}", uuid.clone(), e);
                break;
            }
        };

        if !handler.make_ready().await {
            handler.close();
            handler = ThreadedHandler::new(client_sender.clone()).await;
        }

        if handler.send(msg.into()).await.is_err() {
            break;
        }
    }

    handler.close();

    clients.lock().await.remove(&uuid);
    println!("Client {uuid} Disconnected");
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
        self.sender.blocking_send(Ok(Message::text(result))).unwrap();
    }
}

#[derive(Clone)]
struct CancellableMessage {
    messsage: Message,
    cancel_token: Cancellation,
    completed_token: Arc<AtomicBool>,
}

impl CancellableMessage {
    fn new(messsage: Message) -> Self {
        Self { messsage, cancel_token: Cancellation::new(), completed_token: Arc::new(AtomicBool::from(false)) }
    }
}

impl From<Message> for CancellableMessage {
    fn from(messsage: Message) -> Self {
        Self::new(messsage)
    }
}

struct ThreadedHandler {
    sender: Sender<CancellableMessage>,
    last_message_cancellable: Cancellation,
    last_message_completed: Arc<AtomicBool>,
}

impl ThreadedHandler {
    async fn new(client_sender: Sender<Result<Message, warp::Error>>) -> Self {
        let (handler_sender, mut handler_recv) = mpsc::channel::<CancellableMessage>(5);

        let _ = std::thread::spawn({
            move || {
                let mut message_handler = MessageHandler::new(Box::new(SendResultForWS::new(client_sender)));

                // This is the thread for handling messages from the client.
                // We handle multiple messages before we give up
                while let Some(message) = handler_recv.blocking_recv() {
                    let cancel_token = message.cancel_token;
                    let completed_token = message.completed_token;
                    let message = match message.messsage.to_str() {
                        Ok(v) => v.to_string(),
                        Err(_) => break,
                    };

                    message_handler.handle_message(&message, cancel_token);
                    completed_token.store(true, Ordering::SeqCst);
                }
            }
        });

        ThreadedHandler {
            sender: handler_sender,
            last_message_cancellable: Cancellation::new(),
            last_message_completed: Arc::new(AtomicBool::from(true)),
        }
    }

    /// Returns false if could not make ready in time
    async fn make_ready(&self) -> bool {
        if !self.last_message_completed.load(Ordering::SeqCst) {
            // We're not ready, try and cancel the ongoing operation
            self.last_message_cancellable.cancel();
            let mut counter = 0;
            const MAX_COUNT: usize = 100;
            while counter < MAX_COUNT && !self.last_message_completed.load(Ordering::SeqCst) {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                counter += 1;
            }
            return counter < MAX_COUNT;
        }
        true
    }

    async fn send(&mut self, message: CancellableMessage) -> Result<(), mpsc::error::SendError<CancellableMessage>> {
        self.last_message_cancellable.cancel();
        self.last_message_completed = message.completed_token.clone();
        self.last_message_cancellable = message.cancel_token.clone();
        self.sender.send(message.clone()).await
    }

    fn close(self) {
        let Self { sender, last_message_cancellable, last_message_completed: _ } = self;
        last_message_cancellable.cancel();
        drop(sender);
    }
}

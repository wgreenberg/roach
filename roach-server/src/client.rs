use warp::ws::{WebSocket, Message};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use async_trait::async_trait;

pub struct WebsocketClient {
    pub tx: mpsc::UnboundedSender<String>,
    pub rx: mpsc::UnboundedReceiver<String>,
}

#[derive(PartialEq, Debug)]
pub enum ClientError {
    SendError(String),
    RecvError(String),
}

pub type ClientResult = Result<String, ClientError>;

#[async_trait]
pub trait Client {
    async fn submit_command(&mut self, command: String) -> ClientResult;
}

#[async_trait]
impl Client for WebsocketClient {
    async fn submit_command(&mut self, command: String) -> ClientResult {
        self.tx.send(command.clone())
            .map_err(|err| ClientError::SendError(format!("Couldn't send message {} to client: {}", &command, err)))?;
        self.rx.recv().await
            .ok_or(ClientError::RecvError(format!("Couldn't recieve from client, connection dropped")))
    }
}

impl WebsocketClient {
    pub fn new(socket: WebSocket) -> WebsocketClient {
        let (tx, client_to_ws) = mpsc::unbounded_channel::<String>();
        let (ws_to_client, rx) = mpsc::unbounded_channel::<String>();
        let (ws_tx, mut ws_rx) = socket.split();
        tokio::task::spawn(client_to_ws.map(|s| Ok(Message::text(s)))
            .forward(ws_tx).map(|result| {
                if let Err(e) = result {
                    eprintln!("error sending websocket msg: {}", e);
                }
        }));
        tokio::task::spawn(async move {
            while let Some(result) = ws_rx.next().await {
                let msg = match result {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("error receiving ws message): {}", e);
                        break;
                    }
                };
                match msg.to_str() {
                    Ok(msg_str) => ws_to_client.send(msg_str.to_string())
                        .expect("failed to send message to client"),
                    _ => break,
                };
            }
        });
        WebsocketClient { tx, rx }
    }
}

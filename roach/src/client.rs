use tokio::sync::mpsc;
use async_trait::async_trait;

pub struct WebsocketClient {
    pub tx: mpsc::Sender<String>,
    pub rx: mpsc::Receiver<String>,
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
        self.tx.send(command.clone()).await
            .map_err(|err| ClientError::SendError(format!("Couldn't send message {} to client: {}", &command, err)))?;
        self.rx.recv().await
            .ok_or(ClientError::RecvError(format!("Couldn't recieve from client, connection dropped")))
    }
}

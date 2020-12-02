use reqwest::{Client, Url, Response};
use futures::{StreamExt, SinkExt, FutureExt};
use http::{Request, request::Builder};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use crate::engine::UHPCompliant;

pub struct MatchmakingClient {
    roach_url: Url,
    http_client: Client,
    player_token: String,
}

impl MatchmakingClient {
    pub fn new(roach_server: String, player_token: String) -> Self {
        MatchmakingClient {
            roach_url: Url::parse(&roach_server).expect("failed to parse roach server"),
            http_client: Client::new(),
            player_token,
        }
    }

    pub async fn enter_matchmaking(&self) -> Result<Response, reqwest::Error> {
        self.http_client.post(Url::join(&self.roach_url, "matchmaking").unwrap())
            .header("x-player-auth", &self.player_token)
            .send()
            .await
    }

    async fn poll_matchmaking(&self) -> Result<Response, reqwest::Error> {
        self.http_client.get(Url::join(&self.roach_url, "matchmaking").unwrap())
            .header("x-player-auth", &self.player_token)
            .send()
            .await
    }

    pub async fn wait_for_match(&self) -> Result<i64, reqwest::Error> {
        while let Ok(res) = self.poll_matchmaking().await {
            let obj: serde_json::Value = res.json().await?;
            match &obj["match_info"] {
                serde_json::Value::Object(value) => {
                    println!("{:?}", value);
                    let match_id = value.get("id").unwrap().as_i64().unwrap();
                    return Ok(match_id);
                },
                _ => continue,
            }
        }
        todo!();
    }

    pub async fn play_match(&self, match_id: i64, mut engine: Box<UHPCompliant>) {
        let uri = Url::join(&self.roach_url, &format!("game/{}/play", match_id)).unwrap().into_string();
        let req = Builder::new()
            .uri(uri)
            .header("x-player-auth", &self.player_token)
            .body(()).unwrap();
        let (ws_stream, _) = connect_async(req).await.expect("couldn't connect to websocket endpoint");
        let (mut ws_tx, mut ws_rx) = ws_stream.split();
        while let Some(msg) = ws_rx.next().await {
            let command = msg.unwrap().into_text().expect("couldn't read text from ws message");
            let output = engine.handle_command(&command).await;
            ws_tx.send(Message::text(output)).await;
        }
    }
}

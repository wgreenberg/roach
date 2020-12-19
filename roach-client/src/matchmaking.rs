use reqwest::{Client, Url, Response};
use http::request::Builder;
use tungstenite::{connect, Message};
use crate::engine::UHPCompliant;
use std::{thread, time};

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
            .await?
            .error_for_status()
    }

    async fn poll_matchmaking(&self) -> Result<Response, reqwest::Error> {
        self.http_client.get(Url::join(&self.roach_url, "matchmaking").unwrap())
            .header("x-player-auth", &self.player_token)
            .send()
            .await
    }

    pub async fn wait_for_match(&self) -> Result<(), reqwest::Error> {
        loop {
            let res = self.poll_matchmaking().await?;
            println!("waiting for a match...");
            let status = res.status();
            let obj: serde_json::Value = res.json().await?;
            if status.is_success() {
                if obj["ready"].as_bool().expect("couldn't get ready value") {
                    return Ok(())
                } else {
                    thread::sleep(time::Duration::from_millis(500));
                    continue;
                }
            } else {
                panic!("non-successful status code {} when matchmaking. error: {}", status, obj);
            }
        }
    }

    pub async fn play_match(&self, mut engine: Box<dyn UHPCompliant>) {
        let mut uri = Url::join(&self.roach_url, "play").unwrap();
        uri.set_scheme("wss").expect("couldn't set scheme");
        println!("beginning game {}", &uri);
        let req = Builder::new()
            .uri(uri.into_string())
            .header("x-player-auth", &self.player_token)
            .body(()).unwrap();
        let (mut ws_stream, _) = connect(req).expect("couldn't connect to websocket endpoint");
        while let Ok(msg) = ws_stream.read_message() {
            let command = msg.into_text().expect("couldn't read text from ws message");
            println!("> {}", &command);
            let output = engine.handle_command(&command).await;
            println!("< {}", &output);
            ws_stream.write_message(Message::text(output)).expect("couldn't write message to ws");
        }
    }
}

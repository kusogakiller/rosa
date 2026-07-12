use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;

use reqwest::{Client, Response};

use tokio::{sync::Mutex, time::sleep};

use tracing::info;

use serde_json::Value;

const BASE_URL: &str = "https://zinro.net";

const REQUEST_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Clone)]
pub struct ChatLine {
    pub user: String,
    pub text: String,
    pub color: String,
}

#[derive(Clone)]
pub struct PlayerInfo {
    pub name: String,
    pub job: String,
    pub color: String,
    pub alive: bool,
    pub is_cpu: bool,
}

#[derive(Clone, Default)]
pub struct RoomInfo {
    pub name: String,
    pub scene: String,
    pub day: String,
    pub teiin: String,
}

#[derive(Clone, Default)]
pub struct PlayersSnapshot {
    pub players: Vec<PlayerInfo>,
    pub room: RoomInfo,
    pub me: String,
    pub my_color: String,
}

fn field(obj: &Value, key: &str) -> String {
    obj.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

pub fn parse_messages(raw: &str) -> Vec<ChatLine> {
    let value: Value = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let array = match value.as_array() {
        Some(items) => items,
        None => return Vec::new(),
    };

    let mut lines: Vec<ChatLine> = array
        .iter()
        .map(|item| ChatLine {
            user: field(item, "from_user"),
            text: field(item, "message"),
            color: field(item, "color"),
        })
        .collect();

    lines.reverse();
    lines
}

pub fn parse_players(raw: &str) -> PlayersSnapshot {
    let value: Value = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(_) => return PlayersSnapshot::default(),
    };

    let players = value
        .get("players")
        .and_then(|v| v.as_object())
        .map(|map| {
            map.values()
                .map(|p| PlayerInfo {
                    name: field(p, "name"),
                    job: field(p, "job"),
                    color: field(p, "color"),
                    alive: field(p, "alive") == "1",
                    is_cpu: field(p, "is_cpu") == "1",
                })
                .collect()
        })
        .unwrap_or_default();

    let room = value
        .get("room")
        .map(|r| RoomInfo {
            name: field(r, "name"),
            scene: field(r, "scene"),
            day: field(r, "day"),
            teiin: field(r, "teiin"),
        })
        .unwrap_or_default();

    let me = value
        .get("player")
        .map(|p| field(p, "name"))
        .unwrap_or_default();

    let my_color = value
        .get("player")
        .map(|p| field(p, "color"))
        .unwrap_or_default();

    PlayersSnapshot {
        players,
        room,
        me,
        my_color,
    }
}

pub struct ApiClient {
    client: Client,

    session_key: String,

    last_request: Arc<Mutex<Option<Instant>>>,
}

impl ApiClient {
    pub fn new(session_key: String) -> Result<Self> {
        let client = Client::builder()
            .user_agent("JinroCLI/0.1 (Rust)")
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self {
            client,

            session_key,

            last_request: Arc::new(Mutex::new(None)),
        })
    }

    async fn wait_rate_limit(&self) {
        let wait = {
            let last = self.last_request.lock().await;

            match *last {
                Some(time) => {
                    let elapsed = time.elapsed();

                    if elapsed < REQUEST_INTERVAL {
                        Some(REQUEST_INTERVAL - elapsed)
                    } else {
                        None
                    }
                }

                None => None,
            }
        };

        if let Some(duration) = wait {
            sleep(duration).await;
        }
    }

    async fn update_request_time(&self) {
        let mut last = self.last_request.lock().await;

        *last = Some(Instant::now());
    }

    async fn get(&self, path: &str, params: &[(&str, &str)]) -> Result<Response> {
        self.wait_rate_limit().await;

        let response = self
            .client
            .get(format!("{}{}", BASE_URL, path))
            .query(params)
            .header("Cookie", format!("session_key={}", self.session_key))
            .send()
            .await?;

        self.update_request_time().await;

        info!(status = response.status().as_u16(), "GET request");

        Ok(response)
    }

    async fn post(&self, params: &[(&str, &str)]) -> Result<Response> {
        self.wait_rate_limit().await;

        let response = self
            .client
            .post(format!("{}/m/player.php", BASE_URL))
            .query(params)
            .header("Cookie", format!("session_key={}", self.session_key))
            .send()
            .await?;

        self.update_request_time().await;

        info!(status = response.status().as_u16(), "POST request");

        Ok(response)
    }

    pub async fn send_message(&self, text: &str) -> Result<()> {
        let params = [("mode", "message"), ("to_user", "ALL"), ("message", text)];

        self.post(&params).await?;

        Ok(())
    }

    pub async fn whisper(&self, user: &str, text: &str) -> Result<()> {
        let params = [("mode", "message"), ("to_user", user), ("message", text)];

        self.post(&params).await?;

        Ok(())
    }

    pub async fn get_messages(&self, last_id: Option<&str>) -> Result<String> {
        let mut params = vec![
            ("mode", "message"),
            ("id", "You have good taste, my friend."),
        ];

        if let Some(id) = last_id {
            params.push(("last_id", id));
        }

        Ok(self.get("/m/api/", &params).await?.text().await?)
    }

    pub async fn poll_messages(&self) -> Result<Vec<ChatLine>> {
        let raw = self.get_messages(None).await?;
        Ok(parse_messages(&raw))
    }

    pub async fn poll_players(&self) -> Result<PlayersSnapshot> {
        let raw = self.get_players().await?;
        Ok(parse_players(&raw))
    }

    pub async fn get_players(&self) -> Result<String> {
        let params = [("mode", "players")];

        Ok(self.get("/m/api/", &params).await?.text().await?)
    }
}

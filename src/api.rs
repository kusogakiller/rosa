use std::{
    fs::OpenOptions,
    io::Write,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;

use reqwest::{Client, Response};

use tokio::sync::Mutex;

use tracing::info;

use serde_json::Value;

const BASE_URL: &str = "https://zinro.net";

#[derive(Clone)]
pub struct ChatLine {
    pub id: String,
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
    pub is_active: bool,
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
            id: field(item, "id"),
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
            for p in map.values() {
                let _ = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("rosa_debug.log")
                    .and_then(|mut f| {
                        writeln!(
                            f,
                            "name={} job={:?} is_active={:?}",
                            field(p, "name"),
                            field(p, "job"),
                            p.get("is_active")
                        )
                    });
            }
            map.values()
                .map(|p| PlayerInfo {
                    name: field(p, "name"),
                    job: field(p, "job"),
                    color: field(p, "color"),
                    alive: field(p, "alive") == "1",
                    is_cpu: field(p, "is_cpu") == "1",
                    is_active: p
                        .get("is_active")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
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

    last_message_id: Arc<Mutex<String>>,

    players_cache: Arc<Mutex<Option<(PlayersSnapshot, Instant)>>>,

    last_poll: Arc<Mutex<Option<Instant>>>,

    poll_interval: Arc<Mutex<u128>>,
}

impl ApiClient {
    pub fn new(session_key: String) -> Result<Self> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(5))
            .tcp_keepalive(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(300))
            .build()?;

        Ok(Self {
            client,
            session_key,
        
            last_message_id: Arc::new(Mutex::new(String::new())),
        
            players_cache: Arc::new(Mutex::new(None)),
        
            last_poll: Arc::new(Mutex::new(None)),
        
            poll_interval: Arc::new(Mutex::new(0)),
        })
    }

    async fn get(&self, path: &str, params: &[(&str, &str)]) -> Result<Response> {
        Ok(self
            .client
            .get(format!("{}{}", BASE_URL, path))
            .query(params)
            .header("Cookie", format!("session_key={}", self.session_key))
            .send()
            .await?)
    }

    async fn post(&self, params: &[(&str, &str)]) -> Result<Response> {
        Ok(self
            .client
            .post(format!("{}/m/player.php", BASE_URL))
            .query(params)
            .header("Cookie", format!("session_key={}", self.session_key))
            .send()
            .await?)
    }

    pub async fn send_message(&self, text: &str) -> Result<()> {
        let params = [("mode", "message"), ("to_user", "ALL"), ("message", text)];

        let response = self.post(&params).await?;

        info!(status = response.status().as_u16(), "MESSAGE sent");

        Ok(())
    }

    pub async fn whisper(&self, user: &str, text: &str) -> Result<()> {
        let params = [("mode", "message"), ("to_user", user), ("message", text)];

        let response = self.post(&params).await?;

        info!(
            status = response.status().as_u16(),
            target_user = user,
            "WHISPER sent"
        );

        Ok(())
    }

    pub async fn get_messages(&self) -> Result<Vec<ChatLine>> {
        {
            let mut last = self.last_poll.lock().await;

            if let Some(time) = *last {
                let elapsed = time.elapsed().as_millis();

                let mut interval = self.poll_interval.lock().await;
                *interval = elapsed;
            }

            *last = Some(Instant::now());
        }

        let last_id = {
            let guard = self.last_message_id.lock().await;
            guard.clone()
        };

        let mut params = vec![
            ("mode", "message"),
            ("id", "らぶちぃべんちれーしょんふぉーえばー"),
        ];

        if !last_id.is_empty() {
            params.push(("last_id", &last_id));
        }

        let raw = self.get("/m/api/", &params).await?.text().await?;

        let lines = parse_messages(&raw);

        if let Some(max_id) = lines.iter().map(|x| x.id.as_str()).max() {
            let mut guard = self.last_message_id.lock().await;
            *guard = max_id.to_string();
        }

        Ok(lines)
    }

    pub async fn get_players(&self) -> Result<PlayersSnapshot> {
        const CACHE_TTL: Duration = Duration::from_secs(10);

        {
            let guard = self.players_cache.lock().await;

            if let Some((data, time)) = &*guard {
                if time.elapsed() < CACHE_TTL {
                    return Ok(data.clone());
                }
            }
        }

        let params = [("mode", "players")];

        let raw = self.get("/m/api/", &params).await?.text().await?;

        let data = parse_players(&raw);

        info!(
            player_count = data.players.len(),
            "Players loaded successfully"
        );

        let mut guard = self.players_cache.lock().await;
        *guard = Some((data.clone(), Instant::now()));

        Ok(data)
    }

    pub async fn measured_poll_interval(&self) -> u128 {
        *self.poll_interval.lock().await
    }
}

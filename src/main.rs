mod api;
mod ui;

use std::{io, sync::Arc, time::Duration};

use anyhow::Result;

use api::{ApiClient, ChatLine, PlayerInfo, PlayersSnapshot, RoomInfo};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use ratatui::{Terminal, backend::CrosstermBackend};

#[derive(Clone)]
pub struct Room {
    pub name: String,
}

#[derive(Clone)]
pub struct Message {
    pub user: String,
    pub text: String,
    pub color: String,
    pub pending: bool,
}

pub enum ApiEvent {
    Messages(Vec<ChatLine>),
    Players(PlayersSnapshot),
    Error(String),
}

pub enum ApiCommand {
    Send(String),
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Search,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Panel {
    Explorer,
    Chat,
    Context,
    Terminal,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ExplorerFocus {
    Rooms,
    Users,
}

pub struct App {
    pub api: Arc<ApiClient>,

    pub cmd_tx: UnboundedSender<ApiCommand>,

    pub rooms: Vec<Room>,
    pub selected_room: usize,

    pub users: Vec<PlayerInfo>,
    pub selected_user: usize,

    pub room: RoomInfo,

    pub explorer_focus: ExplorerFocus,

    pub messages: Vec<Message>,
    pub message_cursor: usize,

    pub pending_text: Vec<String>,
    pub my_name: String,
    pub my_color: String,

    pub input: String,
    pub url: String,

    pub command: String,
    pub search: String,

    pub mode: Mode,
    pub panel: Panel,

    pub connected: bool,
    pub running: bool,

    pub pending_g: bool,
    pub pending_d: bool,
    pub pending_ctrl_w: bool,

    pub scroll: usize,

    pub poll_interval_ms: u64,
}

impl App {
    pub fn new(api: Arc<ApiClient>, cmd_tx: UnboundedSender<ApiCommand>) -> Self {
        Self {
            api,

            cmd_tx,

            rooms: Vec::new(),
            selected_room: 0,

            users: Vec::new(),
            selected_user: 0,

            room: RoomInfo::default(),

            explorer_focus: ExplorerFocus::Rooms,

            messages: vec![Message {
                user: "system".into(),
                text: "RosaClient started".into(),
                color: "skyblue".into(),
                pending: false,
            }],

            message_cursor: 0,

            pending_text: Vec::new(),
            my_name: String::new(),
            my_color: String::new(),

            input: String::new(),
            url: String::new(),

            command: String::new(),
            search: String::new(),

            mode: Mode::Normal,
            panel: Panel::Explorer,

            connected: false,
            running: true,

            pending_g: false,
            pending_d: false,
            pending_ctrl_w: false,

            scroll: 0,

            poll_interval_ms: 2000,
        }
    }

    fn apply_event(&mut self, event: ApiEvent) {
        match event {
            ApiEvent::Messages(lines) => {
                self.connected = true;

                if !lines.is_empty() {
                    let was_at_bottom =
                        self.messages.is_empty() || self.message_cursor + 1 >= self.messages.len();

                    let server_new: Vec<Message> = lines
                        .into_iter()
                        .map(|line| Message {
                            user: line.user,
                            text: line.text,
                            color: line.color,
                            pending: false,
                        })
                        .collect();

                    let my = if self.my_name.is_empty() {
                        "me"
                    } else {
                        self.my_name.as_str()
                    };

                    self.pending_text
                        .retain(|text| !server_new.iter().any(|m| m.user == my && m.text == *text));

                    let mut kept: Vec<Message> = self
                        .messages
                        .iter()
                        .filter(|m| m.pending)
                        .cloned()
                        .collect();
                    kept.extend(server_new);

                    for text in self.pending_text.clone() {
                        kept.push(Message {
                            user: if self.my_name.is_empty() {
                                "me".into()
                            } else {
                                self.my_name.clone()
                            },
                            text,
                            color: self.my_color.clone(),
                            pending: true,
                        });
                    }

                    self.messages = kept;

                    let last = self.messages.len().saturating_sub(1);

                    if was_at_bottom {
                        self.message_cursor = last;
                    } else {
                        self.message_cursor = self.message_cursor.min(last);
                    }

                    self.scroll = self.message_cursor;
                }
            }

            ApiEvent::Players(snapshot) => {
                self.connected = true;

                self.users = snapshot.players;
                self.room = snapshot.room;
                self.my_name = snapshot.me;
                self.my_color = snapshot.my_color;

                if !self.room.name.is_empty() {
                    self.rooms = vec![Room {
                        name: self.room.name.clone(),
                    }];
                    self.selected_room = 0;
                }

                self.selected_user = self.selected_user.min(self.users.len().saturating_sub(1));
            }

            ApiEvent::Error(reason) => {
                self.connected = false;

                self.messages.push(Message {
                    user: "system".into(),
                    text: reason,
                    color: "tomato".into(),
                    pending: false,
                });
            }
        }
    }

    fn enter_insert(&mut self) {
        self.panel = Panel::Terminal;
        self.mode = Mode::Insert;
    }

    fn cursor_down(&mut self, step: usize) {
        match self.panel {
            Panel::Explorer => match self.explorer_focus {
                ExplorerFocus::Rooms => {
                    let last = self.rooms.len().saturating_sub(1);
                    self.selected_room = (self.selected_room + step).min(last);
                }
                ExplorerFocus::Users => {
                    let last = self.users.len().saturating_sub(1);
                    self.selected_user = (self.selected_user + step).min(last);
                }
            },
            Panel::Chat => {
                let last = self.messages.len().saturating_sub(1);
                self.message_cursor = (self.message_cursor + step).min(last);
                self.scroll = self.message_cursor;
            }
            _ => {}
        }
    }

    fn cursor_up(&mut self, step: usize) {
        match self.panel {
            Panel::Explorer => match self.explorer_focus {
                ExplorerFocus::Rooms => {
                    self.selected_room = self.selected_room.saturating_sub(step);
                }
                ExplorerFocus::Users => {
                    self.selected_user = self.selected_user.saturating_sub(step);
                }
            },
            Panel::Chat => {
                self.message_cursor = self.message_cursor.saturating_sub(step);
                self.scroll = self.message_cursor;
            }
            _ => {}
        }
    }

    fn goto_top(&mut self) {
        match self.panel {
            Panel::Explorer => match self.explorer_focus {
                ExplorerFocus::Rooms => self.selected_room = 0,
                ExplorerFocus::Users => self.selected_user = 0,
            },
            Panel::Chat => {
                self.message_cursor = 0;
                self.scroll = 0;
            }
            _ => {}
        }
    }

    fn goto_bottom(&mut self) {
        match self.panel {
            Panel::Explorer => match self.explorer_focus {
                ExplorerFocus::Rooms => {
                    self.selected_room = self.rooms.len().saturating_sub(1);
                }
                ExplorerFocus::Users => {
                    self.selected_user = self.users.len().saturating_sub(1);
                }
            },
            Panel::Chat => {
                self.message_cursor = self.messages.len().saturating_sub(1);
                self.scroll = self.message_cursor;
            }
            _ => {}
        }
    }

    fn delete_selected(&mut self) {
        if self.panel == Panel::Chat && self.message_cursor < self.messages.len() {
            let removed = self.messages.remove(self.message_cursor);
            if removed.pending {
                self.pending_text.retain(|t| *t != removed.text);
            }
            self.message_cursor = self
                .message_cursor
                .min(self.messages.len().saturating_sub(1));
            self.scroll = self.message_cursor;
        }
    }

    fn search_jump(&mut self, forward: bool) {
        if self.search.is_empty() || self.messages.is_empty() {
            return;
        }

        let len = self.messages.len();
        let query = self.search.to_lowercase();

        for offset in 1..=len {
            let index = if forward {
                (self.message_cursor + offset) % len
            } else {
                (self.message_cursor + len - offset) % len
            };

            let hit = self.messages[index].text.to_lowercase().contains(&query)
                || self.messages[index].user.to_lowercase().contains(&query);

            if hit {
                self.message_cursor = index;
                self.scroll = index;
                self.panel = Panel::Chat;
                break;
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Normal => self.normal_key(key),
            Mode::Insert => self.insert_key(key),
            Mode::Command => self.command_key(key),
            Mode::Search => self.search_key(key),
        }
    }

    fn normal_key(&mut self, key: KeyEvent) {
        if self.pending_ctrl_w {
            match key.code {
                KeyCode::Char('h') => {
                    self.panel = match self.panel {
                        Panel::Chat => Panel::Explorer,
                        Panel::Terminal => Panel::Context,
                        other => other,
                    };
                }

                KeyCode::Char('l') => {
                    self.panel = match self.panel {
                        Panel::Explorer => Panel::Chat,
                        Panel::Context => Panel::Terminal,
                        other => other,
                    };
                }

                KeyCode::Char('j') => {
                    self.panel = match self.panel {
                        Panel::Explorer => Panel::Context,
                        Panel::Chat => Panel::Terminal,
                        other => other,
                    };
                }

                KeyCode::Char('k') => {
                    self.panel = match self.panel {
                        Panel::Context => Panel::Explorer,
                        Panel::Terminal => Panel::Chat,
                        other => other,
                    };
                }

                _ => {}
            }

            self.pending_ctrl_w = false;
            return;
        }

        if self.pending_g {
            if key.code == KeyCode::Char('g') {
                self.goto_top();
            }

            self.pending_g = false;
            return;
        }

        if self.pending_d {
            if key.code == KeyCode::Char('d') {
                self.delete_selected();
            }

            self.pending_d = false;
            return;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('w') => {
                    self.pending_ctrl_w = true;
                    return;
                }
                KeyCode::Char('d') => {
                    self.cursor_down(8);
                    return;
                }
                KeyCode::Char('u') => {
                    self.cursor_up(8);
                    return;
                }
                KeyCode::Char('f') => {
                    self.cursor_down(16);
                    return;
                }
                KeyCode::Char('b') => {
                    self.cursor_up(16);
                    return;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char('q') => {
                self.running = false;
            }

            KeyCode::Char('i')
            | KeyCode::Char('a')
            | KeyCode::Char('A')
            | KeyCode::Char('I')
            | KeyCode::Char('o')
            | KeyCode::Char('O') => {
                self.enter_insert();
            }

            KeyCode::Char(':') => {
                self.command.clear();

                self.mode = Mode::Command;
            }

            KeyCode::Char('/') => {
                self.search.clear();

                self.mode = Mode::Search;
            }

            KeyCode::Char('n') => {
                self.search_jump(true);
            }

            KeyCode::Char('N') => {
                self.search_jump(false);
            }

            KeyCode::Char('d') => {
                self.pending_d = true;
            }

            KeyCode::Char('x') => {
                self.delete_selected();
            }

            KeyCode::Char('g') => {
                self.pending_g = true;
            }

            KeyCode::Char('G') => {
                self.goto_bottom();
            }

            KeyCode::Tab => {
                self.panel = match self.panel {
                    Panel::Explorer => Panel::Chat,
                    Panel::Chat => Panel::Context,
                    Panel::Context => Panel::Terminal,
                    Panel::Terminal => Panel::Explorer,
                };
            }

            KeyCode::BackTab => {
                self.panel = match self.panel {
                    Panel::Explorer => Panel::Terminal,
                    Panel::Chat => Panel::Explorer,
                    Panel::Context => Panel::Chat,
                    Panel::Terminal => Panel::Context,
                };
            }

            KeyCode::Char('j') | KeyCode::Down => {
                self.cursor_down(1);
            }

            KeyCode::Char('k') | KeyCode::Up => {
                self.cursor_up(1);
            }

            KeyCode::Char('h') | KeyCode::Left => {
                if self.panel == Panel::Explorer {
                    self.explorer_focus = ExplorerFocus::Rooms;
                }
            }

            KeyCode::Char('l') | KeyCode::Right => {
                if self.panel == Panel::Explorer {
                    self.explorer_focus = ExplorerFocus::Users;
                }
            }

            KeyCode::Esc => {
                self.pending_g = false;
                self.pending_d = false;
                self.pending_ctrl_w = false;
                self.mode = Mode::Normal;
            }

            _ => {}
        }
    }

    fn insert_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }

            KeyCode::Enter => {
                if !self.input.is_empty() {
                    let text = self.input.clone();

                    let _ = self.cmd_tx.send(ApiCommand::Send(text.clone()));

                    self.pending_text.push(text.clone());

                    self.messages.push(Message {
                        user: if self.my_name.is_empty() {
                            "me".into()
                        } else {
                            self.my_name.clone()
                        },
                        text,
                        color: self.my_color.clone(),
                        pending: true,
                    });

                    self.message_cursor = self.messages.len().saturating_sub(1);

                    self.input.clear();
                }
            }

            KeyCode::Backspace => {
                self.input.pop();
            }

            KeyCode::Char(c) => {
                self.input.push(c);
            }

            _ => {}
        }
    }

    fn command_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }

            KeyCode::Enter => {
                match self.command.as_str() {
                    "q" | "quit" => {
                        self.running = false;
                    }

                    "clear" => {
                        self.messages.clear();
                        self.message_cursor = 0;
                    }

                    _ => {}
                }

                self.command.clear();
                self.mode = Mode::Normal;
            }

            KeyCode::Backspace => {
                self.command.pop();
            }

            KeyCode::Char(c) => {
                self.command.push(c);
            }

            _ => {}
        }
    }

    fn search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }

            KeyCode::Enter => {
                self.mode = Mode::Normal;
                self.search_jump(true);
            }

            KeyCode::Backspace => {
                self.search.pop();
            }

            KeyCode::Char(c) => {
                self.search.push(c);
            }

            _ => {}
        }
    }
}

fn spawn_poll_task(api: Arc<ApiClient>, tx: UnboundedSender<ApiEvent>, interval_ms: u64) {
    tokio::spawn(async move {
        let interval = Duration::from_millis(interval_ms);

        loop {
            match api.poll_messages().await {
                Ok(lines) => {
                    if !lines.is_empty() {
                        let _ = tx.send(ApiEvent::Messages(lines));
                    }
                }
                Err(err) => {
                    api.mark_failure().await;
                    let _ = tx.send(ApiEvent::Error(err.to_string()));
                }
            }

            match api.poll_players().await {
                Ok(Some(users)) => {
                    let _ = tx.send(ApiEvent::Players(users));
                }
                Ok(None) => {}
                Err(err) => {
                    api.mark_failure().await;
                    let _ = tx.send(ApiEvent::Error(err.to_string()));
                }
            }

            let delay = api.next_poll_delay(interval).await;
            tokio::time::sleep(delay).await;
        }
    });
}

fn spawn_command_task(
    api: Arc<ApiClient>,
    mut rx: UnboundedReceiver<ApiCommand>,
    tx: UnboundedSender<ApiEvent>,
) {
    tokio::spawn(async move {
        while let Some(command) = rx.recv().await {
            match command {
                ApiCommand::Send(text) => {
                    if let Err(err) = api.send_message(&text).await {
                        let _ = tx.send(ApiEvent::Error(err.to_string()));
                    }
                }
            }
        }
    });
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let session_key = std::env::var("SESSION_KEY").expect("SESSION_KEY is not set");

    let api = Arc::new(ApiClient::new(session_key)?);

    let (event_tx, mut event_rx) = unbounded_channel::<ApiEvent>();

    let (cmd_tx, cmd_rx) = unbounded_channel::<ApiCommand>();

    spawn_poll_task(api.clone(), event_tx.clone(), 2000);

    spawn_command_task(api.clone(), cmd_rx, event_tx.clone());

    crossterm::terminal::enable_raw_mode()?;

    let mut stdout = io::stdout();

    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(api, cmd_tx);

    while app.running {
        while let Ok(event) = event_rx.try_recv() {
            app.apply_event(event);
        }

        terminal.draw(|frame| {
            ui::draw(frame, &app);
        })?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
        }
    }

    crossterm::terminal::disable_raw_mode()?;

    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;

    Ok(())
}

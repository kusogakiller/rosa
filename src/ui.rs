use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Padding, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
};

use crate::{App, ExplorerFocus, Mode, Panel};

struct Theme;

impl Theme {
    const BASE: Color = Color::Rgb(30, 30, 46);
    const MANTLE: Color = Color::Rgb(24, 24, 37);
    const SURFACE: Color = Color::Rgb(49, 50, 68);
    const OVERLAY: Color = Color::Rgb(108, 112, 134);
    const TEXT: Color = Color::Rgb(205, 214, 244);
    const SUBTEXT: Color = Color::Rgb(166, 173, 200);
    const PINK: Color = Color::Rgb(245, 194, 231);
    const MAUVE: Color = Color::Rgb(203, 166, 247);
    const GREEN: Color = Color::Rgb(166, 227, 161);
    const RED: Color = Color::Rgb(243, 139, 168);
    const PEACH: Color = Color::Rgb(250, 179, 135);
    const BLUE: Color = Color::Rgb(137, 180, 250);
}

const SEP_RIGHT: &str = "\u{e0b0}";
const SEP_LEFT: &str = "\u{e0b2}";

fn chat_color(name: &str) -> Color {
    if let Some(hex) = name.strip_prefix('#') {
        let hex = match hex.len() {
            3 => hex.chars().flat_map(|c| [c, c]).collect::<String>(),
            _ => hex.to_string(),
        };
        if hex.len() == 6 {
            if let Ok(rgb) = u32::from_str_radix(&hex, 16) {
                return Color::Rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8);
            }
        }
    }

    match name {
        "white" => Color::Rgb(205, 214, 244),
        "tomato" => Color::Rgb(243, 139, 168),
        "lime" => Color::Rgb(166, 227, 161),
        "tan" => Color::Rgb(250, 179, 135),
        "violet" => Color::Rgb(203, 166, 247),
        "pink" => Color::Rgb(245, 194, 231),
        "skyblue" => Color::Rgb(137, 220, 235),
        _ => Theme::TEXT,
    }
}

fn panel_block(icon: &str, title: &str, focused: bool) -> Block<'static> {
    let border = if focused {
        Theme::MAUVE
    } else {
        Theme::SURFACE
    };

    let (dot, title_style) = if focused {
        (
            Span::styled("\u{25cf} ", Style::default().fg(Theme::MAUVE)),
            Style::default()
                .fg(Theme::MAUVE)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (
            Span::styled("\u{25cb} ", Style::default().fg(Theme::OVERLAY)),
            Style::default().fg(Theme::SUBTEXT),
        )
    };

    let title = Line::from(vec![
        Span::raw(" "),
        dot,
        Span::styled(format!("{}  {} ", icon, title), title_style),
    ]);

    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(Theme::BASE))
}

pub fn draw(frame: &mut Frame, app: &App) {
    if frame.area().height < 14 || frame.area().width < 35 {
        return;
    }

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(34), Constraint::Min(1)])
        .split(root[1]);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(10)])
        .split(main[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(4)])
        .split(main[1]);

    draw_topbar(frame, app, root[0]);
    draw_explorer(frame, app, left[0]);
    draw_context(frame, app, left[1]);
    draw_chat(frame, app, right[0]);
    draw_terminal(frame, app, right[1]);
    draw_status(frame, app, root[2]);
}

fn draw_topbar(frame: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(16)])
        .split(area);

    let mut left = vec![
        Span::styled(
            " \u{f4d5} RosaClient ",
            Style::default()
                .fg(Theme::BASE)
                .bg(Theme::MAUVE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            SEP_RIGHT,
            Style::default().fg(Theme::MAUVE).bg(Theme::MANTLE),
        ),
        Span::raw(" "),
    ];

    for (panel, label) in [
        (Panel::Explorer, "Explorer"),
        (Panel::Chat, "Chat"),
        (Panel::Context, "Context"),
        (Panel::Terminal, "Terminal"),
    ] {
        let active = app.panel == panel;
        let style = if active {
            Style::default()
                .fg(Theme::PINK)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::OVERLAY)
        };
        let marker = if active { "\u{2022} " } else { "  " };
        left.push(Span::styled(format!("{}{}  ", marker, label), style));
    }

    let title = Paragraph::new(Line::from(left)).style(Style::default().bg(Theme::MANTLE));
    frame.render_widget(title, cols[0]);

    let (dot, text, color) = if app.connected {
        ("\u{25cf}", "online", Theme::GREEN)
    } else {
        ("\u{25cf}", "offline", Theme::RED)
    };

    let status = Paragraph::new(Line::from(vec![
        Span::styled(format!("{} ", dot), Style::default().fg(color)),
        Span::styled(format!("{} ", text), Style::default().fg(Theme::SUBTEXT)),
    ]))
    .alignment(Alignment::Right)
    .style(Style::default().bg(Theme::MANTLE));
    frame.render_widget(status, cols[1]);
}

fn draw_explorer(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.panel == Panel::Explorer;

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(1)])
        .split(area);

    let rooms_active = focused && app.explorer_focus == ExplorerFocus::Rooms;
    let users_active = focused && app.explorer_focus == ExplorerFocus::Users;

    let rooms: Vec<ListItem> = if app.rooms.is_empty() {
        vec![ListItem::new(Span::styled(
            "connecting...",
            Style::default()
                .fg(Theme::OVERLAY)
                .add_modifier(Modifier::ITALIC),
        ))]
    } else {
        app.rooms
            .iter()
            .map(|room| {
                ListItem::new(Line::from(vec![
                    Span::styled("\u{f0a0a} ", Style::default().fg(Theme::PEACH)),
                    Span::styled(room.name.clone(), Style::default().fg(Theme::TEXT)),
                ]))
            })
            .collect()
    };

    let mut rooms_state = ListState::default();
    if !app.rooms.is_empty() {
        rooms_state.select(Some(app.selected_room.min(app.rooms.len() - 1)));
    }

    let rooms_widget = List::new(rooms)
        .block(panel_block("\u{f0219}", "Rooms", rooms_active))
        .highlight_style(
            Style::default()
                .bg(Theme::SURFACE)
                .fg(Theme::PINK)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("\u{2503} ");
    frame.render_stateful_widget(rooms_widget, inner[0], &mut rooms_state);

    let users: Vec<ListItem> = if app.users.is_empty() {
        vec![ListItem::new(Span::styled(
            "waiting for players...",
            Style::default()
                .fg(Theme::OVERLAY)
                .add_modifier(Modifier::ITALIC),
        ))]
    } else {
        app.users
            .iter()
            .map(|user| {
                let is_spectator = user.job == "\u{89b3}\u{6218}\u{8005}";

                let (icon, icon_color) = if is_spectator {
                    ("\u{f09ce}", Theme::PINK)
                } else if !user.alive {
                    ("\u{f0159}", Theme::OVERLAY)
                } else if user.is_cpu {
                    ("\u{f12b1}", Theme::BLUE)
                } else {
                    ("\u{f04b}", Theme::GREEN)
                };

                let name_style = if is_spectator {
                    Style::default().fg(Theme::PINK)
                } else if user.alive {
                    Style::default().fg(chat_color(&user.color))
                } else {
                    Style::default()
                        .fg(Theme::OVERLAY)
                        .add_modifier(Modifier::CROSSED_OUT)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", icon), Style::default().fg(icon_color)),
                    Span::styled(user.name.clone(), name_style),
                    Span::styled(
                        format!("  {}", user.job),
                        Style::default().fg(Theme::OVERLAY),
                    ),
                ]))
            })
            .collect()
    };

    let mut users_state = ListState::default();
    if !app.users.is_empty() {
        users_state.select(Some(app.selected_user.min(app.users.len() - 1)));
    }

    let users_widget = List::new(users)
        .block(panel_block("\u{f0849}", "Users", users_active))
        .highlight_style(
            Style::default()
                .bg(Theme::SURFACE)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("\u{2503} ");
    frame.render_stateful_widget(users_widget, inner[1], &mut users_state);
}

fn draw_chat(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.panel == Panel::Chat;

    let items: Vec<ListItem> = app
        .messages
        .iter()
        .map(|message| {
            let base = chat_color(&message.color);
            let status = if message.pending {
                Span::styled(
                    "  \u{f252} sending\u{2026}",
                    Style::default()
                        .fg(Theme::OVERLAY)
                        .add_modifier(Modifier::ITALIC),
                )
            } else {
                Span::styled("", Style::default())
            };
            let line = Line::from(vec![
                Span::styled("\u{25cf} ", Style::default().fg(base)),
                Span::styled(
                    message.user.clone(),
                    Style::default().fg(base).add_modifier(Modifier::BOLD),
                ),
                Span::styled("  ", Style::default()),
                Span::styled(message.text.clone(), Style::default().fg(Theme::TEXT)),
                status,
            ]);
            ListItem::new(line)
        })
        .collect();

    let mut state = ListState::default();
    if !app.messages.is_empty() {
        state.select(Some(app.message_cursor.min(app.messages.len() - 1)));
    }

    let widget = List::new(items)
        .block(panel_block("\u{f4b9}", "Chat", focused))
        .highlight_style(
            Style::default()
                .bg(Theme::SURFACE)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("\u{2503} ");
    frame.render_stateful_widget(widget, area, &mut state);

    if app.messages.len() > area.height.saturating_sub(2) as usize {
        let mut sb_state = ScrollbarState::new(app.messages.len()).position(app.message_cursor);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .thumb_style(Style::default().fg(Theme::OVERLAY))
            .track_style(Style::default().fg(Theme::MANTLE));
        frame.render_stateful_widget(scrollbar, area, &mut sb_state);
    }
}

fn draw_context(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.panel == Panel::Context;

    let (conn_text, conn_color) = if app.connected {
        ("\u{25cf} connected", Theme::GREEN)
    } else {
        ("\u{25cf} offline", Theme::RED)
    };

    let room = if app.room.name.is_empty() {
        "-"
    } else {
        app.room.name.as_str()
    };

    let alive = app.users.iter().filter(|u| u.alive).count();

    let scene = if app.room.scene.is_empty() {
        "-"
    } else {
        app.room.scene.as_str()
    };

    let label = |s: &str| Span::styled(format!("{:<8}", s), Style::default().fg(Theme::OVERLAY));

    let lines = vec![
        Line::from(vec![
            label("session"),
            Span::styled(
                conn_text,
                Style::default().fg(conn_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            label("room"),
            Span::styled(room, Style::default().fg(Theme::TEXT)),
        ]),
        Line::from(vec![
            label("scene"),
            Span::styled(
                scene,
                Style::default()
                    .fg(Theme::MAUVE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  day {}", app.room.day),
                Style::default().fg(Theme::SUBTEXT),
            ),
        ]),
        Line::from(vec![
            label("players"),
            Span::styled(
                format!("{}/{}", alive, app.users.len()),
                Style::default()
                    .fg(Theme::PEACH)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  capacity {}", app.room.teiin),
                Style::default().fg(Theme::SUBTEXT),
            ),
        ]),
        Line::from(vec![
            label("polling"),
            Span::styled(
                format!("{} ms", app.poll_interval_ms),
                Style::default().fg(Theme::TEXT),
            ),
        ]),
    ];

    let widget = Paragraph::new(lines)
        .block(panel_block("\u{f05a9}", "Context", focused))
        .wrap(Wrap { trim: true });
    frame.render_widget(widget, area);
}

fn draw_terminal(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.panel == Panel::Terminal;

    let (prompt, prompt_color, body) = match app.mode {
        Mode::Command => (":", Theme::PEACH, app.command.clone()),
        Mode::Search => ("/", Theme::PINK, app.search.clone()),
        _ => ("\u{276f}", Theme::GREEN, app.input.clone()),
    };

    let cursor = if focused && app.mode != Mode::Normal {
        "\u{2588}"
    } else {
        ""
    };

    let line = Line::from(vec![
        Span::styled(
            format!("{} ", prompt),
            Style::default()
                .fg(prompt_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(body, Style::default().fg(Theme::TEXT)),
        Span::styled(
            cursor,
            Style::default()
                .fg(Theme::PINK)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
    ]);

    let widget = Paragraph::new(line).block(panel_block("\u{f489}", "Terminal", focused));
    frame.render_widget(widget, area);
}

fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let (mode, mode_color) = match app.mode {
        Mode::Normal => ("NORMAL", Theme::MAUVE),
        Mode::Insert => ("INSERT", Theme::GREEN),
        Mode::Command => ("COMMAND", Theme::PEACH),
        Mode::Search => ("SEARCH", Theme::PINK),
    };

    let panel = match app.panel {
        Panel::Explorer => "explorer",
        Panel::Chat => "chat",
        Panel::Context => "context",
        Panel::Terminal => "terminal",
    };

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(30)])
        .split(area);

    let left = Line::from(vec![
        Span::styled(
            format!(" {} ", mode),
            Style::default()
                .fg(Theme::BASE)
                .bg(mode_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            SEP_RIGHT,
            Style::default().fg(mode_color).bg(Theme::SURFACE),
        ),
        Span::styled(
            format!(" \u{f4d5} {} ", panel),
            Style::default().fg(Theme::TEXT).bg(Theme::SURFACE),
        ),
        Span::styled(
            SEP_RIGHT,
            Style::default().fg(Theme::SURFACE).bg(Theme::MANTLE),
        ),
    ]);

    let room = if app.room.name.is_empty() {
        "no room".to_string()
    } else {
        app.room.name.clone()
    };

    let right = Line::from(vec![
        Span::styled(
            SEP_LEFT,
            Style::default().fg(Theme::SURFACE).bg(Theme::MANTLE),
        ),
        Span::styled(
            format!(" \u{f0219} {} ", room),
            Style::default().fg(Theme::TEXT).bg(Theme::SURFACE),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(left).style(Style::default().bg(Theme::MANTLE)),
        cols[0],
    );
    frame.render_widget(
        Paragraph::new(right)
            .alignment(Alignment::Right)
            .style(Style::default().bg(Theme::MANTLE)),
        cols[1],
    );
}

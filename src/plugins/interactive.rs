use crate::plugins::Plugin;
use crate::steam_client::{Achievement, AchievementSet, Game, SteamClient, SteamError};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use ratatui::crossterm::ExecutableCommand;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Gauge, List, ListItem, ListState, Paragraph, Row, Table, TableState,
    },
};
use std::collections::HashMap;
use std::io::{Write, stdout};
use std::sync::Arc;
use tokio::sync::mpsc;
use unicode_width::UnicodeWidthStr;

pub struct InteractivePlugin;

#[async_trait]
impl Plugin for InteractivePlugin {
    fn command(&self) -> clap::Command {
        clap::Command::new("interactive").about("Interactive TUI mode")
    }

    async fn execute(
        &self,
        _steam: &dyn SteamClient,
        _matches: &clap::ArgMatches,
        _writer: &mut (dyn Write + Send),
        _err_writer: &mut (dyn Write + Send),
    ) {
        unreachable!("Interactive plugin is handled directly in main.rs")
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Screen {
    List,
    Detail(u32),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AchievementViewMode {
    All,
    Remaining,
    Unlocked,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AchievementSortMode {
    Name,
    UnlockDate,
    GlobalPercent,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    fn toggle(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

pub struct State {
    pub games: Vec<Game>,
    pub filter: String,
    pub selection_idx: usize,
    pub screen: Screen,
    pub view_mode: AchievementViewMode,
    pub sort_mode: AchievementSortMode,
    pub sort_dir: SortDirection,
    pub achievements_cache: HashMap<u32, Result<AchievementSet, SteamError>>,
    pub loading_achievements: bool,
    pub detail_selection_idx: usize,
}

#[derive(Debug, PartialEq)]
pub enum Effect {
    None,
    Quit,
    FetchAchievements(u32),
}

impl State {
    pub fn new(mut games: Vec<Game>) -> Self {
        games.sort_by(|a, b| {
            b.rtime_last_played
                .cmp(&a.rtime_last_played)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        Self {
            games,
            filter: String::new(),
            selection_idx: 0,
            screen: Screen::List,
            view_mode: AchievementViewMode::All,
            sort_mode: AchievementSortMode::Name,
            sort_dir: SortDirection::Ascending,
            achievements_cache: HashMap::new(),
            loading_achievements: false,
            detail_selection_idx: 0,
        }
    }

    pub fn filtered_games(&self) -> Vec<&Game> {
        let filter_lower = self.filter.to_lowercase();
        self.games
            .iter()
            .filter(|g| g.name.to_lowercase().contains(&filter_lower))
            .collect()
    }

    pub fn visible_achievements(&self, appid: u32) -> Option<Vec<&Achievement>> {
        let set = match self.achievements_cache.get(&appid) {
            Some(Ok(s)) => s,
            _ => return None,
        };

        let mut achs: Vec<_> = set
            .achievements
            .iter()
            .filter(|a| match self.view_mode {
                AchievementViewMode::All => true,
                AchievementViewMode::Remaining => a.achieved == 0,
                AchievementViewMode::Unlocked => a.achieved > 0,
            })
            .collect();

        achs.sort_by(|a, b| {
            let cmp = match self.sort_mode {
                AchievementSortMode::Name => a.name.cmp(&b.name),
                AchievementSortMode::UnlockDate => a
                    .unlocktime
                    .cmp(&b.unlocktime)
                    .then_with(|| a.name.cmp(&b.name)),
                AchievementSortMode::GlobalPercent => {
                    let pa = a.global_percent.unwrap_or(0.0);
                    let pb = b.global_percent.unwrap_or(0.0);
                    pa.partial_cmp(&pb)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.name.cmp(&b.name))
                }
            };
            match self.sort_dir {
                SortDirection::Ascending => cmp,
                SortDirection::Descending => cmp.reverse(),
            }
        });
        Some(achs)
    }

    pub fn clamp_detail_selection(&mut self, appid: u32) {
        if let Some(achs) = self.visible_achievements(appid) {
            let max_idx = achs.len().saturating_sub(1);
            if self.detail_selection_idx > max_idx {
                self.detail_selection_idx = max_idx;
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Effect {
        match self.screen {
            Screen::List => self.handle_list_key(key),
            Screen::Detail(appid) => self.handle_detail_key(key, appid),
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> Effect {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Effect::Quit;
        }

        let filtered = self.filtered_games();
        let max_idx = filtered.len().saturating_sub(1);

        match key.code {
            KeyCode::Esc => {
                let has_filter = !self.filter.is_empty();
                drop(filtered);
                if has_filter {
                    self.filter.clear();
                    self.selection_idx = 0;
                } else {
                    return Effect::Quit;
                }
            }
            KeyCode::Enter => {
                let appid = filtered.get(self.selection_idx).map(|g| g.appid);
                drop(filtered);
                if let Some(appid) = appid {
                    self.screen = Screen::Detail(appid);
                    self.loading_achievements = true;
                    self.detail_selection_idx = 0;
                    if !self.achievements_cache.contains_key(&appid) {
                        return Effect::FetchAchievements(appid);
                    } else {
                        self.loading_achievements = false;
                    }
                }
            }
            KeyCode::Up => {
                self.selection_idx = self.selection_idx.saturating_sub(1);
            }
            KeyCode::Down => {
                if self.selection_idx < max_idx {
                    self.selection_idx += 1;
                }
            }
            KeyCode::PageUp => {
                self.selection_idx = self.selection_idx.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.selection_idx = std::cmp::min(self.selection_idx + 10, max_idx);
            }
            KeyCode::Backspace => {
                if self.filter.pop().is_some() {
                    self.selection_idx = 0;
                }
            }
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.filter.push(c);
                self.selection_idx = 0;
            }
            _ => {}
        }
        Effect::None
    }

    fn handle_detail_key(&mut self, key: KeyEvent, appid: u32) -> Effect {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Effect::Quit;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                self.screen = Screen::List;
            }
            KeyCode::Char('a') => {
                self.view_mode = AchievementViewMode::All;
                self.clamp_detail_selection(appid);
            }
            KeyCode::Char('r') => {
                self.view_mode = AchievementViewMode::Remaining;
                self.clamp_detail_selection(appid);
            }
            KeyCode::Char('u') => {
                self.view_mode = AchievementViewMode::Unlocked;
                self.clamp_detail_selection(appid);
            }
            KeyCode::Char('n') => {
                if self.sort_mode == AchievementSortMode::Name {
                    self.sort_dir = self.sort_dir.toggle();
                } else {
                    self.sort_mode = AchievementSortMode::Name;
                    self.sort_dir = SortDirection::Ascending;
                }
            }
            KeyCode::Char('d') => {
                if self.sort_mode == AchievementSortMode::UnlockDate {
                    self.sort_dir = self.sort_dir.toggle();
                } else {
                    self.sort_mode = AchievementSortMode::UnlockDate;
                    self.sort_dir = SortDirection::Descending;
                }
            }
            KeyCode::Char('g') => {
                if self.sort_mode == AchievementSortMode::GlobalPercent {
                    self.sort_dir = self.sort_dir.toggle();
                } else {
                    self.sort_mode = AchievementSortMode::GlobalPercent;
                    self.sort_dir = SortDirection::Descending;
                }
            }
            KeyCode::Up => {
                self.detail_selection_idx = self.detail_selection_idx.saturating_sub(1);
            }
            KeyCode::Down => {
                self.detail_selection_idx += 1;
                self.clamp_detail_selection(appid);
            }
            KeyCode::PageUp => {
                self.detail_selection_idx = self.detail_selection_idx.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.detail_selection_idx += 10;
                self.clamp_detail_selection(appid);
            }
            _ => {}
        }
        Effect::None
    }
}

pub fn render(state: &State, frame: &mut Frame) {
    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(frame.area());

    match state.screen {
        Screen::List => render_list(state, frame, rects[0]),
        Screen::Detail(appid) => render_detail(state, appid, frame, rects[0]),
    }

    render_footer(state, frame, rects[1]);
}

/// Builds a single game list line with the game name left-aligned and the date right-aligned.
///
/// <purpose-start>
/// Extracts the line-building logic from render_list so it can be unit-tested independently.
/// The caller supplies the inner width (total area width minus border characters) to ensure
/// the date string is not clipped by the border.
/// </purpose-end>
///
/// <inputs-start>
/// - `game_name`: The display name of the game.
/// - `date_str`: The formatted date string (e.g. "2026-07-30" or "Never").
/// - `inner_width`: The usable character width inside the bordered area (area.width - 2).
/// </inputs-start>
///
/// <outputs-start>
/// - A `Line` containing three spans: game name, padding spaces, and date string.
/// </outputs-start>
///
/// <side-effects-start>
/// - None.
/// </side-effects-end>
pub fn build_game_list_line<'a>(game_name: &str, date_str: &str, inner_width: u16) -> Line<'a> {
    let name_len = game_name.width() as u16;
    let date_len = date_str.width() as u16;
    let padding = inner_width.saturating_sub(name_len + date_len) as usize;
    Line::from(vec![
        Span::raw(game_name.to_owned()),
        Span::raw(" ".repeat(padding)),
        Span::raw(date_str.to_owned()),
    ])
}

fn render_list(state: &State, frame: &mut Frame, area: Rect) {
    let filtered = state.filtered_games();
    // Subtract 2 for the left and right border characters so the content
    // fits within the visible inner area and the date is not clipped.
    let inner_width = area.width.saturating_sub(2);
    let mut items = Vec::new();

    for game in &filtered {
        let date_str = if game.rtime_last_played > 0 {
            Utc.timestamp_opt(game.rtime_last_played as i64, 0)
                .unwrap()
                .format("%Y-%m-%d")
                .to_string()
        } else {
            "Never".to_string()
        };
        let line = build_game_list_line(&game.name, &date_str, inner_width);
        items.push(ListItem::new(line));
    }

    if items.is_empty() {
        let msg = format!("No games match '{}'", state.filter);
        frame.render_widget(
            Paragraph::new(msg).block(Block::default().borders(Borders::ALL)),
            area,
        );
        return;
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Owned Games (Filter: {}) ", state.filter)),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut list_state = ListState::default();
    list_state.select(Some(state.selection_idx));

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_detail(state: &State, appid: u32, frame: &mut Frame, area: Rect) {
    if state.loading_achievements {
        frame.render_widget(
            Paragraph::new("Loading...").block(Block::default().borders(Borders::ALL)),
            area,
        );
        return;
    }

    let result = match state.achievements_cache.get(&appid) {
        Some(res) => res,
        None => return,
    };

    match result {
        Err(SteamError::NoStats { .. }) => {
            frame.render_widget(
                Paragraph::new("This game has no achievements")
                    .block(Block::default().borders(Borders::ALL)),
                area,
            );
        }
        Err(e) => {
            let msg = e.to_string();
            frame.render_widget(
                Paragraph::new(msg)
                    .style(Style::default().fg(Color::Red))
                    .block(Block::default().borders(Borders::ALL).title("Error")),
                area,
            );
        }
        Ok(set) => {
            let total = set.achievements.len();
            let unlocked = set.achievements.iter().filter(|a| a.achieved > 0).count();
            let percent = if total > 0 {
                (unlocked as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3), // Progress bar
                        Constraint::Min(0),    // Table
                        Constraint::Length(3), // Description
                    ]
                    .as_ref(),
                )
                .split(area);

            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .title(set.game_name.clone())
                        .borders(Borders::ALL),
                )
                .gauge_style(Style::default().fg(Color::Green))
                .percent(percent as u16)
                .label(format!("{} / {} ({:.1}%)", unlocked, total, percent));
            frame.render_widget(gauge, chunks[0]);

            if let Some(achs) = state.visible_achievements(appid) {
                let rows: Vec<Row> = achs
                    .iter()
                    .map(|a| {
                        let status = if a.achieved > 0 { "✓" } else { "·" };
                        let status_style = if a.achieved > 0 {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default().add_modifier(Modifier::DIM)
                        };
                        let date_str = if a.achieved > 0 {
                            Utc.timestamp_opt(a.unlocktime as i64, 0)
                                .unwrap()
                                .format("%Y-%m-%d")
                                .to_string()
                        } else {
                            "".to_string()
                        };
                        let percent_str = if let Some(p) = a.global_percent {
                            format!("{:.1}%", p)
                        } else {
                            "".to_string()
                        };

                        Row::new(vec![
                            Cell::from(Span::styled(status, status_style)),
                            Cell::from(a.name.clone()),
                            Cell::from(date_str),
                            Cell::from(percent_str),
                        ])
                    })
                    .collect();

                let table = Table::new(
                    rows,
                    [
                        Constraint::Length(3),
                        Constraint::Percentage(60),
                        Constraint::Length(12),
                        Constraint::Length(10),
                    ],
                )
                .header(
                    Row::new(vec!["St", "Name", "Unlocked", "Global %"])
                        .style(Style::default().add_modifier(Modifier::BOLD)),
                )
                .block(Block::default().borders(Borders::ALL))
                .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

                let mut table_state = TableState::default();
                table_state.select(Some(state.detail_selection_idx));
                frame.render_stateful_widget(table, chunks[1], &mut table_state);

                let desc = if let Some(a) = achs.get(state.detail_selection_idx) {
                    a.description.clone()
                } else {
                    String::new()
                };
                frame.render_widget(
                    Paragraph::new(desc)
                        .block(Block::default().borders(Borders::ALL).title("Description")),
                    chunks[2],
                );
            }
        }
    }
}

fn render_footer(state: &State, frame: &mut Frame, area: Rect) {
    let text = match state.screen {
        Screen::List => {
            "Type to filter | ↑/↓: Move | Enter: View | Esc: Clear/Quit | Ctrl-C: Quit".to_string()
        }
        Screen::Detail(_) => {
            let view = match state.view_mode {
                AchievementViewMode::All => "All",
                AchievementViewMode::Remaining => "Rem",
                AchievementViewMode::Unlocked => "Unl",
            };
            let sort = match state.sort_mode {
                AchievementSortMode::Name => "Name",
                AchievementSortMode::UnlockDate => "Date",
                AchievementSortMode::GlobalPercent => "Global%",
            };
            let dir = match state.sort_dir {
                SortDirection::Ascending => "Asc",
                SortDirection::Descending => "Desc",
            };
            format!(
                "Esc/q: Back | a/r/u: View({}) | n/d/g: Sort({} {}) | ↑/↓: Scroll",
                view, sort, dir
            )
        }
    };
    frame.render_widget(
        Paragraph::new(text).style(Style::default().add_modifier(Modifier::REVERSED)),
        area,
    );
}

pub enum AppEvent {
    Key(KeyEvent),
    Achievements {
        appid: u32,
        result: Result<AchievementSet, SteamError>,
    },
}

pub async fn run(steam: Arc<dyn SteamClient>) {
    // Check terminal
    let mut stdout = stdout();
    use std::io::IsTerminal;
    if !stdout.is_terminal() {
        eprintln!("Interactive mode requires a terminal.");
        std::process::exit(1);
    }

    let games_res = steam.owned_games().await;
    let games = match games_res {
        Ok(g) if !g.is_empty() => g,
        Ok(_) => {
            eprintln!("No games found in your library.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to load games: {}", e);
            std::process::exit(1);
        }
    };

    enable_raw_mode().unwrap();
    stdout.execute(EnterAlternateScreen).unwrap();
    std::panic::set_hook(Box::new(|info| {
        let _ = disable_raw_mode();
        let _ = std::io::stdout().execute(LeaveAlternateScreen);
        eprintln!("{}", info);
    }));

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout)).unwrap();
    let mut state = State::new(games);

    let (tx, mut rx) = mpsc::unbounded_channel();
    let tx_key = tx.clone();

    std::thread::spawn(move || {
        loop {
            if let Ok(Event::Key(key)) = event::read()
                && key.kind == event::KeyEventKind::Press
                && tx_key.send(AppEvent::Key(key)).is_err()
            {
                break;
            }
        }
    });

    loop {
        terminal.draw(|f| render(&state, f)).unwrap();

        if let Some(ev) = rx.recv().await {
            match ev {
                AppEvent::Key(key) => {
                    let effect = state.handle_key(key);
                    match effect {
                        Effect::Quit => break,
                        Effect::FetchAchievements(appid) => {
                            let tx = tx.clone();
                            let steam = steam.clone();
                            tokio::spawn(async move {
                                // Fetches achievements enriched with global unlock
                                // percentages via the centralized trait method, avoiding
                                // duplicating the apiname-join logic inline.
                                let result = steam.achievements_with_global(appid).await;
                                let _ = tx.send(AppEvent::Achievements { appid, result });
                            });
                        }
                        Effect::None => {}
                    }
                }
                AppEvent::Achievements { appid, result } => {
                    state.achievements_cache.insert(appid, result);
                    if let Screen::Detail(current_appid) = state.screen
                        && current_appid == appid
                    {
                        state.loading_achievements = false;
                    }
                }
            }
        }
    }

    let _ = disable_raw_mode();
    let _ = terminal.backend_mut().execute(LeaveAlternateScreen);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    fn make_game(appid: u32, name: &str, rtime_last_played: u64) -> Game {
        Game {
            appid,
            name: name.to_string(),
            playtime_forever: 0,
            img_icon_url: String::new(),
            playtime_windows_forever: 0,
            playtime_mac_forever: 0,
            playtime_linux_forever: 0,
            rtime_last_played,
            playtime_disconnected: 0,
        }
    }

    fn make_achievement(
        apiname: &str,
        name: &str,
        achieved: u8,
        unlocktime: u64,
        global_percent: Option<f32>,
    ) -> Achievement {
        Achievement {
            apiname: apiname.to_string(),
            achieved,
            unlocktime,
            name: name.to_string(),
            description: format!("{name} description"),
            global_percent,
        }
    }

    fn make_achievement_set() -> AchievementSet {
        AchievementSet {
            game_name: "Test Game".to_string(),
            achievements: vec![
                make_achievement("a_alpha", "Alpha", 1, 200, Some(10.0)),
                make_achievement("b_beta", "Beta", 0, 0, None),
                make_achievement("c_gamma", "Gamma", 1, 100, Some(50.0)),
            ],
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn key_with(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn test_state_new() {
        let g1 = make_game(1, "B", 10);
        let g2 = make_game(2, "A", 20);
        let state = State::new(vec![g1, g2]);
        assert_eq!(state.games[0].name, "A"); // 20 > 10
    }

    /// Verifies that `build_game_list_line` produces a line whose total character
    /// width matches `inner_width` and includes the complete date string, ensuring
    /// the date is never clipped by the border.
    #[test]
    fn test_build_game_list_line_date_not_truncated() {
        let game_name = "Cyberpunk 2077";
        let date_str = "2026-07-30";
        // Simulate a terminal area.width of 100 => inner_width = 98 (minus 2 for borders)
        let inner_width: u16 = 98;

        let line = build_game_list_line(game_name, date_str, inner_width);
        let total_len: usize = line.spans.iter().map(|s| s.content.len()).sum();

        assert_eq!(
            total_len, inner_width as usize,
            "Line width must equal inner_width"
        );
        // The last span must be the complete date string
        assert_eq!(
            line.spans.last().unwrap().content,
            date_str,
            "Date must not be truncated"
        );
    }

    /// Verifies padding is zero (no extra spaces) when the game name + date fill
    /// or exceed the available inner width.
    #[test]
    fn test_build_game_list_line_no_padding_when_full() {
        let game_name = "A Very Long Game Name That Fills The Width Completely";
        let date_str = "2026-07-30";
        // Inner width smaller than name + date
        let inner_width: u16 = 50;

        let line = build_game_list_line(game_name, date_str, inner_width);
        // Padding span should be empty
        assert_eq!(
            line.spans[1].content.as_ref(),
            "",
            "Padding must be empty when content overflows"
        );
        // Date must still be fully present
        assert_eq!(
            line.spans.last().unwrap().content,
            date_str,
            "Date must not be truncated"
        );
    }

    /// Verifies that game names containing multi-byte unicode characters (such as trademark symbols ™)
    /// calculate padding based on display width (terminal columns) rather than byte count.
    #[test]
    fn test_build_game_list_line_with_unicode_symbols() {
        let game_name = "Middle-earth™: Shadow of War™";
        let date_str = "2024-07-13";
        let inner_width: u16 = 90;

        let line = build_game_list_line(game_name, date_str, inner_width);

        let name_display_width = game_name.width();
        let date_display_width = date_str.width();
        let padding_spaces = line.spans[1].content.len();

        assert_eq!(
            name_display_width + padding_spaces + date_display_width,
            inner_width as usize,
            "Total visual width (name width + padding + date width) must match inner_width"
        );
    }

    #[test]
    fn test_state_new_tiebreak_on_name() {
        let g1 = make_game(1, "Zebra", 10);
        let g2 = make_game(2, "apple", 10);
        let state = State::new(vec![g1, g2]);
        // Equal rtime_last_played falls back to case-insensitive name order.
        assert_eq!(state.games[0].name, "apple");
        assert_eq!(state.games[1].name, "Zebra");
    }

    #[test]
    fn test_filtered_games() {
        let state = State::new(vec![
            make_game(1, "Half-Life", 0),
            make_game(2, "Portal", 0),
        ]);
        assert_eq!(state.filtered_games().len(), 2);

        let mut state = state;
        state.filter = "life".to_string();
        let filtered = state.filtered_games();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Half-Life");

        state.filter = "nomatch".to_string();
        assert!(state.filtered_games().is_empty());
    }

    #[test]
    fn test_visible_achievements_none_when_not_cached() {
        let state = State::new(vec![make_game(1, "Game", 0)]);
        assert!(state.visible_achievements(1).is_none());
    }

    #[test]
    fn test_visible_achievements_none_when_cached_error() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state
            .achievements_cache
            .insert(1, Err(SteamError::PrivateProfile));
        assert!(state.visible_achievements(1).is_none());
    }

    #[test]
    fn test_visible_achievements_view_modes() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state
            .achievements_cache
            .insert(1, Ok(make_achievement_set()));

        state.view_mode = AchievementViewMode::All;
        assert_eq!(state.visible_achievements(1).unwrap().len(), 3);

        state.view_mode = AchievementViewMode::Remaining;
        let remaining = state.visible_achievements(1).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].apiname, "b_beta");

        state.view_mode = AchievementViewMode::Unlocked;
        let unlocked = state.visible_achievements(1).unwrap();
        assert_eq!(unlocked.len(), 2);
    }

    #[test]
    fn test_visible_achievements_sort_name() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state
            .achievements_cache
            .insert(1, Ok(make_achievement_set()));
        state.sort_mode = AchievementSortMode::Name;

        state.sort_dir = SortDirection::Ascending;
        let asc = state.visible_achievements(1).unwrap();
        assert_eq!(asc[0].name, "Alpha");
        assert_eq!(asc[2].name, "Gamma");

        state.sort_dir = SortDirection::Descending;
        let desc = state.visible_achievements(1).unwrap();
        assert_eq!(desc[0].name, "Gamma");
        assert_eq!(desc[2].name, "Alpha");
    }

    #[test]
    fn test_visible_achievements_sort_unlock_date() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state
            .achievements_cache
            .insert(1, Ok(make_achievement_set()));
        state.sort_mode = AchievementSortMode::UnlockDate;
        state.sort_dir = SortDirection::Ascending;

        let asc = state.visible_achievements(1).unwrap();
        // Beta (unlocktime 0) < Gamma (100) < Alpha (200)
        assert_eq!(asc[0].name, "Beta");
        assert_eq!(asc[1].name, "Gamma");
        assert_eq!(asc[2].name, "Alpha");
    }

    #[test]
    fn test_visible_achievements_sort_global_percent() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state
            .achievements_cache
            .insert(1, Ok(make_achievement_set()));
        state.sort_mode = AchievementSortMode::GlobalPercent;
        state.sort_dir = SortDirection::Ascending;

        // Beta has no global percent -> treated as 0.0, sorts first ascending.
        let asc = state.visible_achievements(1).unwrap();
        assert_eq!(asc[0].name, "Beta");
        assert_eq!(asc[1].name, "Alpha");
        assert_eq!(asc[2].name, "Gamma");
    }

    #[test]
    fn test_clamp_detail_selection() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state
            .achievements_cache
            .insert(1, Ok(make_achievement_set()));

        state.detail_selection_idx = 1;
        state.clamp_detail_selection(1);
        assert_eq!(state.detail_selection_idx, 1); // within bounds, unchanged

        state.detail_selection_idx = 99;
        state.clamp_detail_selection(1);
        assert_eq!(state.detail_selection_idx, 2); // clamped to max_idx (3 items)
    }

    #[test]
    fn test_handle_list_key_ctrl_c_quits() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        let effect = state.handle_key(key_with(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert_eq!(effect, Effect::Quit);
    }

    #[test]
    fn test_handle_list_key_esc_clears_filter_then_quits() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state.filter = "abc".to_string();
        state.selection_idx = 2;

        let effect = state.handle_key(key(KeyCode::Esc));
        assert_eq!(effect, Effect::None);
        assert_eq!(state.filter, "");
        assert_eq!(state.selection_idx, 0);

        let effect = state.handle_key(key(KeyCode::Esc));
        assert_eq!(effect, Effect::Quit);
    }

    #[test]
    fn test_handle_list_key_enter_cache_miss_fetches() {
        let mut state = State::new(vec![make_game(42, "Game", 0)]);
        let effect = state.handle_key(key(KeyCode::Enter));
        assert_eq!(effect, Effect::FetchAchievements(42));
        assert_eq!(state.screen, Screen::Detail(42));
        assert!(state.loading_achievements);
    }

    #[test]
    fn test_handle_list_key_enter_cache_hit_no_fetch() {
        let mut state = State::new(vec![make_game(42, "Game", 0)]);
        state
            .achievements_cache
            .insert(42, Ok(make_achievement_set()));

        let effect = state.handle_key(key(KeyCode::Enter));
        assert_eq!(effect, Effect::None);
        assert_eq!(state.screen, Screen::Detail(42));
        assert!(!state.loading_achievements);
    }

    #[test]
    fn test_handle_list_key_enter_no_games_is_noop() {
        let mut state = State::new(vec![]);
        let effect = state.handle_key(key(KeyCode::Enter));
        assert_eq!(effect, Effect::None);
        assert_eq!(state.screen, Screen::List);
    }

    #[test]
    fn test_handle_list_key_navigation() {
        let mut state = State::new(vec![
            make_game(1, "A", 0),
            make_game(2, "B", 0),
            make_game(3, "C", 0),
        ]);

        state.handle_key(key(KeyCode::Up));
        assert_eq!(state.selection_idx, 0); // saturating, stays at 0

        state.handle_key(key(KeyCode::Down));
        assert_eq!(state.selection_idx, 1);

        state.handle_key(key(KeyCode::PageDown));
        assert_eq!(state.selection_idx, 2); // clamped to max_idx

        state.handle_key(key(KeyCode::PageUp));
        assert_eq!(state.selection_idx, 0); // saturating_sub(10)
    }

    #[test]
    fn test_handle_list_key_backspace_and_char() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state.selection_idx = 5;

        state.handle_key(key(KeyCode::Char('g')));
        assert_eq!(state.filter, "g");
        assert_eq!(state.selection_idx, 0);

        state.selection_idx = 3;
        state.handle_key(key(KeyCode::Backspace));
        assert_eq!(state.filter, "");
        assert_eq!(state.selection_idx, 0);
    }

    #[test]
    fn test_handle_list_key_ignores_modified_chars() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state.handle_key(key_with(KeyCode::Char('x'), KeyModifiers::ALT));
        assert_eq!(state.filter, "");
    }

    #[test]
    fn test_handle_detail_key_ctrl_c_quits() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state.screen = Screen::Detail(1);
        let effect = state.handle_key(key_with(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert_eq!(effect, Effect::Quit);
    }

    #[test]
    fn test_handle_detail_key_back_to_list() {
        for code in [KeyCode::Esc, KeyCode::Char('q'), KeyCode::Backspace] {
            let mut state = State::new(vec![make_game(1, "Game", 0)]);
            state.screen = Screen::Detail(1);
            state.handle_key(key(code));
            assert_eq!(state.screen, Screen::List);
        }
    }

    #[test]
    fn test_handle_detail_key_view_modes() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state.screen = Screen::Detail(1);
        state
            .achievements_cache
            .insert(1, Ok(make_achievement_set()));

        state.handle_key(key(KeyCode::Char('r')));
        assert_eq!(state.view_mode, AchievementViewMode::Remaining);

        state.handle_key(key(KeyCode::Char('u')));
        assert_eq!(state.view_mode, AchievementViewMode::Unlocked);

        state.handle_key(key(KeyCode::Char('a')));
        assert_eq!(state.view_mode, AchievementViewMode::All);
    }

    #[test]
    fn test_handle_detail_key_sort_modes_set_then_toggle() {
        for (code, mode, first_dir) in [
            (
                KeyCode::Char('n'),
                AchievementSortMode::Name,
                SortDirection::Ascending,
            ),
            (
                KeyCode::Char('d'),
                AchievementSortMode::UnlockDate,
                SortDirection::Descending,
            ),
            (
                KeyCode::Char('g'),
                AchievementSortMode::GlobalPercent,
                SortDirection::Descending,
            ),
        ] {
            let mut state = State::new(vec![make_game(1, "Game", 0)]);
            state.screen = Screen::Detail(1);
            // State::new defaults sort_mode to Name; start from a different
            // mode so the first press always exercises the "set" branch.
            state.sort_mode = if mode == AchievementSortMode::UnlockDate {
                AchievementSortMode::Name
            } else {
                AchievementSortMode::UnlockDate
            };
            state.sort_dir = SortDirection::Ascending;

            state.handle_key(key(code));
            assert_eq!(state.sort_mode, mode);
            assert_eq!(state.sort_dir, first_dir);

            // Second press on the same mode toggles direction instead of resetting it.
            state.handle_key(key(code));
            assert_eq!(state.sort_mode, mode);
            assert_ne!(state.sort_dir, first_dir);
        }
    }

    #[test]
    fn test_handle_detail_key_navigation() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state.screen = Screen::Detail(1);
        state
            .achievements_cache
            .insert(1, Ok(make_achievement_set()));

        state.handle_key(key(KeyCode::Down));
        assert_eq!(state.detail_selection_idx, 1);

        state.handle_key(key(KeyCode::Up));
        assert_eq!(state.detail_selection_idx, 0);

        state.handle_key(key(KeyCode::PageDown));
        assert_eq!(state.detail_selection_idx, 2); // clamped to 3 achievements

        state.handle_key(key(KeyCode::PageUp));
        assert_eq!(state.detail_selection_idx, 0);
    }

    #[test]
    fn test_render_list_normal() {
        let state = State::new(vec![make_game(1, "Portal", 0)]);
        let mut terminal = Terminal::new(TestBackend::new(60, 10)).unwrap();
        terminal.draw(|f| render(&state, f)).unwrap();
        let content = terminal.backend().to_string();
        assert!(content.contains("Portal"));
        assert!(content.contains("Type to filter"));
    }

    #[test]
    fn test_render_list_no_matches() {
        let mut state = State::new(vec![make_game(1, "Portal", 0)]);
        state.filter = "zzz".to_string();
        let mut terminal = Terminal::new(TestBackend::new(60, 10)).unwrap();
        terminal.draw(|f| render(&state, f)).unwrap();
        let content = terminal.backend().to_string();
        assert!(content.contains("No games match"));
    }

    #[test]
    fn test_render_detail_loading() {
        let mut state = State::new(vec![make_game(1, "Portal", 0)]);
        state.screen = Screen::Detail(1);
        state.loading_achievements = true;
        let mut terminal = Terminal::new(TestBackend::new(60, 10)).unwrap();
        terminal.draw(|f| render(&state, f)).unwrap();
        let content = terminal.backend().to_string();
        assert!(content.contains("Loading"));
    }

    #[test]
    fn test_render_detail_cache_miss_renders_nothing_extra() {
        let mut state = State::new(vec![make_game(1, "Portal", 0)]);
        state.screen = Screen::Detail(1);
        let mut terminal = Terminal::new(TestBackend::new(60, 10)).unwrap();
        // Should not panic even though nothing is cached for appid 1.
        terminal.draw(|f| render(&state, f)).unwrap();
    }

    #[test]
    fn test_render_detail_no_stats() {
        let mut state = State::new(vec![make_game(1, "Portal", 0)]);
        state.screen = Screen::Detail(1);
        state
            .achievements_cache
            .insert(1, Err(SteamError::NoStats { appid: 1 }));
        let mut terminal = Terminal::new(TestBackend::new(60, 10)).unwrap();
        terminal.draw(|f| render(&state, f)).unwrap();
        let content = terminal.backend().to_string();
        assert!(content.contains("no achievements"));
    }

    #[test]
    fn test_render_detail_generic_error() {
        let mut state = State::new(vec![make_game(1, "Portal", 0)]);
        state.screen = Screen::Detail(1);
        state
            .achievements_cache
            .insert(1, Err(SteamError::PrivateProfile));
        let mut terminal = Terminal::new(TestBackend::new(60, 10)).unwrap();
        terminal.draw(|f| render(&state, f)).unwrap();
        let content = terminal.backend().to_string();
        assert!(content.contains("private"));
    }

    #[test]
    fn test_render_detail_ok_set() {
        let mut state = State::new(vec![make_game(1, "Portal", 0)]);
        state.screen = Screen::Detail(1);
        state
            .achievements_cache
            .insert(1, Ok(make_achievement_set()));
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal.draw(|f| render(&state, f)).unwrap();
        let content = terminal.backend().to_string();
        assert!(content.contains("Alpha"));
        assert!(content.contains("Test Game"));
    }

    #[test]
    fn test_render_footer_labels_per_sort_mode() {
        for (mode, sort_label) in [
            (AchievementSortMode::Name, "Name"),
            (AchievementSortMode::UnlockDate, "Date"),
            (AchievementSortMode::GlobalPercent, "Global%"),
        ] {
            let mut state = State::new(vec![make_game(1, "Portal", 0)]);
            state.screen = Screen::Detail(1);
            state.sort_mode = mode;
            let mut terminal = Terminal::new(TestBackend::new(80, 10)).unwrap();
            terminal.draw(|f| render(&state, f)).unwrap();
            let content = terminal.backend().to_string();
            assert!(content.contains(sort_label));
        }
    }
}

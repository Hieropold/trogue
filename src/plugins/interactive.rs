use crate::plugins::Plugin;
use crate::steam_client::{Achievement, AchievementSet, Game, SteamClient, SteamError};
use async_trait::async_trait;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::crossterm::ExecutableCommand;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, TableState, List, ListItem, ListState},
    Frame, Terminal,
};
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::sync::Arc;
use tokio::sync::mpsc;
use chrono::{TimeZone, Utc};
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

    pub fn visible_achievements<'a>(&'a self, appid: u32) -> Option<Vec<&'a Achievement>> {
        let set = match self.achievements_cache.get(&appid) {
            Some(Ok(s)) => s,
            _ => return None,
        };

        let mut achs: Vec<_> = set.achievements.iter().filter(|a| {
            match self.view_mode {
                AchievementViewMode::All => true,
                AchievementViewMode::Remaining => a.achieved == 0,
                AchievementViewMode::Unlocked => a.achieved > 0,
            }
        }).collect();

        achs.sort_by(|a, b| {
            let cmp = match self.sort_mode {
                AchievementSortMode::Name => a.name.cmp(&b.name),
                AchievementSortMode::UnlockDate => {
                    a.unlocktime.cmp(&b.unlocktime).then_with(|| a.name.cmp(&b.name))
                },
                AchievementSortMode::GlobalPercent => {
                    let pa = a.global_percent.unwrap_or(0.0);
                    let pb = b.global_percent.unwrap_or(0.0);
                    pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.name.cmp(&b.name))
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
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) => {
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
            Utc.timestamp_opt(game.rtime_last_played as i64, 0).unwrap().format("%Y-%m-%d").to_string()
        } else {
            "Never".to_string()
        };
        let line = build_game_list_line(&game.name, &date_str, inner_width);
        items.push(ListItem::new(line));
    }

    if items.is_empty() {
        let msg = format!("No games match '{}'", state.filter);
        frame.render_widget(Paragraph::new(msg).block(Block::default().borders(Borders::ALL)), area);
        return;
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(" Owned Games (Filter: {}) ", state.filter)))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut list_state = ListState::default();
    list_state.select(Some(state.selection_idx));

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_detail(state: &State, appid: u32, frame: &mut Frame, area: Rect) {
    if state.loading_achievements {
        frame.render_widget(Paragraph::new("Loading...").block(Block::default().borders(Borders::ALL)), area);
        return;
    }

    let result = match state.achievements_cache.get(&appid) {
        Some(res) => res,
        None => return,
    };

    match result {
        Err(SteamError::NoStats { .. }) => {
            frame.render_widget(Paragraph::new("This game has no achievements").block(Block::default().borders(Borders::ALL)), area);
            return;
        }
        Err(e) => {
            let msg = e.to_string();
            frame.render_widget(
                Paragraph::new(msg)
                    .style(Style::default().fg(Color::Red))
                    .block(Block::default().borders(Borders::ALL).title("Error")),
                area,
            );
            return;
        }
        Ok(set) => {
            let total = set.achievements.len();
            let unlocked = set.achievements.iter().filter(|a| a.achieved > 0).count();
            let percent = if total > 0 { (unlocked as f64 / total as f64) * 100.0 } else { 0.0 };

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Progress bar
                    Constraint::Min(0),    // Table
                    Constraint::Length(3), // Description
                ].as_ref())
                .split(area);

            let gauge = Gauge::default()
                .block(Block::default().title(set.game_name.clone()).borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Green))
                .percent(percent as u16)
                .label(format!("{} / {} ({:.1}%)", unlocked, total, percent));
            frame.render_widget(gauge, chunks[0]);

            if let Some(achs) = state.visible_achievements(appid) {
                let rows: Vec<Row> = achs.iter().map(|a| {
                    let status = if a.achieved > 0 { "✓" } else { "·" };
                    let status_style = if a.achieved > 0 { Style::default().fg(Color::Green) } else { Style::default().add_modifier(Modifier::DIM) };
                    let date_str = if a.achieved > 0 {
                        Utc.timestamp_opt(a.unlocktime as i64, 0).unwrap().format("%Y-%m-%d").to_string()
                    } else {
                        "".to_string()
                    };
                    let percent_str = if let Some(p) = a.global_percent { format!("{:.1}%", p) } else { "".to_string() };
                    
                    Row::new(vec![
                        Cell::from(Span::styled(status, status_style)),
                        Cell::from(a.name.clone()),
                        Cell::from(date_str),
                        Cell::from(percent_str),
                    ])
                }).collect();

                let table = Table::new(rows, [Constraint::Length(3), Constraint::Percentage(60), Constraint::Length(12), Constraint::Length(10)])
                    .header(Row::new(vec!["St", "Name", "Unlocked", "Global %"]).style(Style::default().add_modifier(Modifier::BOLD)))
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
                frame.render_widget(Paragraph::new(desc).block(Block::default().borders(Borders::ALL).title("Description")), chunks[2]);
            }
        }
    }
}

fn render_footer(state: &State, frame: &mut Frame, area: Rect) {
    let text = match state.screen {
        Screen::List => "Type to filter | ↑/↓: Move | Enter: View | Esc: Clear/Quit | Ctrl-C: Quit".to_string(),
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
            format!("Esc/q: Back | a/r/u: View({}) | n/d/g: Sort({} {}) | ↑/↓: Scroll", view, sort, dir)
        }
    };
    frame.render_widget(Paragraph::new(text).style(Style::default().add_modifier(Modifier::REVERSED)), area);
}

pub enum AppEvent {
    Key(KeyEvent),
    Achievements { appid: u32, result: Result<AchievementSet, SteamError> },
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
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == event::KeyEventKind::Press {
                    if tx_key.send(AppEvent::Key(key)).is_err() {
                        break;
                    }
                }
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
                                // For global percentages
                                let result = async {
                                    let mut set = steam.achievements(appid).await?;
                                    if let Ok(globals) = steam.global_percentages(appid).await {
                                        for a in &mut set.achievements {
                                            if let Some(g) = globals.iter().find(|x| x.name == a.apiname) {
                                                a.global_percent = Some(g.percent);
                                            }
                                        }
                                    }
                                    Ok(set)
                                }.await;
                                let _ = tx.send(AppEvent::Achievements { appid, result });
                            });
                        }
                        Effect::None => {}
                    }
                }
                AppEvent::Achievements { appid, result } => {
                    state.achievements_cache.insert(appid, result);
                    if let Screen::Detail(current_appid) = state.screen {
                        if current_appid == appid {
                            state.loading_achievements = false;
                        }
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

    #[test]
    fn test_state_new() {
        let g1 = Game { appid: 1, name: "B".to_string(), playtime_forever: 0, img_icon_url: String::new(), playtime_windows_forever: 0, playtime_mac_forever: 0, playtime_linux_forever: 0, rtime_last_played: 10, playtime_disconnected: 0 };
        let g2 = Game { appid: 2, name: "A".to_string(), playtime_forever: 0, img_icon_url: String::new(), playtime_windows_forever: 0, playtime_mac_forever: 0, playtime_linux_forever: 0, rtime_last_played: 20, playtime_disconnected: 0 };
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

        assert_eq!(total_len, inner_width as usize, "Line width must equal inner_width");
        // The last span must be the complete date string
        assert_eq!(line.spans.last().unwrap().content, date_str, "Date must not be truncated");
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
        assert_eq!(line.spans[1].content.as_ref(), "", "Padding must be empty when content overflows");
        // Date must still be fully present
        assert_eq!(line.spans.last().unwrap().content, date_str, "Date must not be truncated");
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
}

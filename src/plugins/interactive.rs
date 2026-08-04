use crate::game_library::{
    Achievement, AchievementSet, Game, GameId, GameLibrary, Platform, PlatformError,
};
use crate::plugins::Plugin;
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
        clap::Command::new("interactive").about("Run interactive TUI mode")
    }

    async fn execute(
        &self,
        _steam: &dyn GameLibrary,
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
    Detail(GameId),
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
    pub achievements_cache: HashMap<GameId, Result<AchievementSet, PlatformError>>,
    pub loading_achievements: bool,
    pub detail_selection_idx: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Effect {
    None,
    Quit,
    FetchAchievements(GameId),
}

impl State {
    pub fn new(mut games: Vec<Game>) -> Self {
        games.sort_by(|a, b| {
            b.rtime_last_played
                .cmp(&a.rtime_last_played)
                .then_with(|| a.name.cmp(&b.name))
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
        if self.filter.is_empty() {
            return self.games.iter().collect();
        }
        let filter_lower = self.filter.to_lowercase();
        self.games
            .iter()
            .filter(|g| g.name.to_lowercase().contains(&filter_lower))
            .collect()
    }

    pub fn visible_achievements(&self, id: &GameId) -> Option<Vec<&Achievement>> {
        let set = match self.achievements_cache.get(id) {
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

    pub fn clamp_detail_selection(&mut self, id: &GameId) {
        if let Some(achs) = self.visible_achievements(id) {
            let max_idx = achs.len().saturating_sub(1);
            if self.detail_selection_idx > max_idx {
                self.detail_selection_idx = max_idx;
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Effect {
        match self.screen.clone() {
            Screen::List => self.handle_list_key(key),
            Screen::Detail(id) => self.handle_detail_key(key, id),
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
                let id = filtered.get(self.selection_idx).map(|g| g.id.clone());
                drop(filtered);
                if let Some(id) = id {
                    self.screen = Screen::Detail(id.clone());
                    self.loading_achievements = true;
                    self.detail_selection_idx = 0;
                    if !self.achievements_cache.contains_key(&id) {
                        return Effect::FetchAchievements(id);
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

    fn handle_detail_key(&mut self, key: KeyEvent, id: GameId) -> Effect {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Effect::Quit;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                self.screen = Screen::List;
            }
            KeyCode::Char('a') => {
                self.view_mode = AchievementViewMode::All;
                self.clamp_detail_selection(&id);
            }
            KeyCode::Char('r') => {
                self.view_mode = AchievementViewMode::Remaining;
                self.clamp_detail_selection(&id);
            }
            KeyCode::Char('u') => {
                self.view_mode = AchievementViewMode::Unlocked;
                self.clamp_detail_selection(&id);
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
                self.clamp_detail_selection(&id);
            }
            KeyCode::PageUp => {
                self.detail_selection_idx = self.detail_selection_idx.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.detail_selection_idx += 10;
                self.clamp_detail_selection(&id);
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

    match &state.screen {
        Screen::List => render_list(state, frame, rects[0]),
        Screen::Detail(id) => render_detail(state, id, frame, rects[0]),
    }

    render_footer(state, frame, rects[1]);
}

/// Builds a single game list line with the game name left-aligned and the date right-aligned.
///
/// <purpose-start>
/// Formats a single row in the game list, ensuring proper spacing and preventing clipping.
/// <purpose-end>
///
/// <inputs-start>
/// - `game_name`: The name of the game.
/// - `date_str`: Formatted last played date string.
/// - `inner_width`: Total available inner width.
/// <inputs-end>
///
/// <outputs-start>
/// - `Line<'static>`: Renderable line widget item.
/// <outputs-end>
///
/// <side-effects-start>
/// - None.
/// <side-effects-end>
fn build_game_list_line(game_name: &str, date_str: &str, inner_width: u16) -> Line<'static> {
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

fn render_detail(state: &State, id: &GameId, frame: &mut Frame, area: Rect) {
    if state.loading_achievements && !state.achievements_cache.contains_key(id) {
        frame.render_widget(
            Paragraph::new("Loading...").block(Block::default().borders(Borders::ALL)),
            area,
        );
        return;
    }

    let result = match state.achievements_cache.get(id) {
        Some(res) => res,
        None => return,
    };

    match result {
        Err(PlatformError::NoStats { .. }) => {
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
            let completed = set.achievements.iter().filter(|a| a.achieved > 0).count();
            let ratio = if total > 0 {
                completed as f64 / total as f64
            } else {
                0.0
            };
            let percent_str = format!("{:.1}% ({}/{})", ratio * 100.0, completed, total);

            let visible = state.visible_achievements(id).unwrap_or_default();

            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(4), Constraint::Min(0)].as_ref())
                .split(area);

            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" {} ", set.game_name)),
                )
                .gauge_style(Style::default().fg(Color::Green))
                .ratio(ratio)
                .label(percent_str);

            frame.render_widget(gauge, main_chunks[0]);

            let rows: Vec<Row> = visible
                .iter()
                .map(|a| {
                    let status = if a.achieved > 0 {
                        Span::styled("Unlocked", Style::default().fg(Color::Green))
                    } else {
                        Span::styled("Locked", Style::default().fg(Color::DarkGray))
                    };

                    let date_str = if a.unlocktime > 0 {
                        Utc.timestamp_opt(a.unlocktime as i64, 0)
                            .unwrap()
                            .format("%Y-%m-%d %H:%M")
                            .to_string()
                    } else {
                        "-".to_string()
                    };

                    let global_str = match a.global_percent {
                        Some(p) => format!("{:.1}%", p),
                        None => "-".to_string(),
                    };

                    Row::new(vec![
                        Cell::from(status),
                        Cell::from(a.name.clone()),
                        Cell::from(a.description.clone()),
                        Cell::from(date_str),
                        Cell::from(global_str),
                    ])
                })
                .collect();

            let header = Row::new(vec![
                "Status",
                "Name",
                "Description",
                "Unlocked",
                "Global %",
            ])
            .style(Style::default().add_modifier(Modifier::BOLD));

            let table = Table::new(
                rows,
                [
                    Constraint::Length(10),
                    Constraint::Percentage(25),
                    Constraint::Percentage(45),
                    Constraint::Length(18),
                    Constraint::Length(10),
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Achievements "),
            )
            .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

            let mut table_state = TableState::default();
            table_state.select(Some(state.detail_selection_idx));

            frame.render_stateful_widget(table, main_chunks[1], &mut table_state);
        }
    }
}

fn render_footer(state: &State, frame: &mut Frame, area: Rect) {
    let text = match state.screen {
        Screen::List => {
            if state.filter.is_empty() {
                " [Esc/Ctrl+C] Quit | [Enter] Details | Type to filter ".to_string()
            } else {
                format!(
                    " [Esc] Clear Filter ({}) | [Ctrl+C] Quit | [Enter] Details ",
                    state.filter
                )
            }
        }
        Screen::Detail(_) => {
            let view_str = match state.view_mode {
                AchievementViewMode::All => "All",
                AchievementViewMode::Remaining => "Remaining",
                AchievementViewMode::Unlocked => "Unlocked",
            };
            let sort_str = match state.sort_mode {
                AchievementSortMode::Name => "Name",
                AchievementSortMode::UnlockDate => "Date",
                AchievementSortMode::GlobalPercent => "Global%",
            };
            let dir_str = match state.sort_dir {
                SortDirection::Ascending => "Asc",
                SortDirection::Descending => "Desc",
            };
            format!(
                " [Esc/q] Back | [a]ll [r]emaining [u]nlocked ({view_str}) | Sort: [n]ame [d]ate [g]lobal ({sort_str} {dir_str}) "
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
        id: GameId,
        result: Result<AchievementSet, PlatformError>,
    },
}

pub async fn run(steam: Arc<dyn GameLibrary>) {
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
                        Effect::FetchAchievements(id) => {
                            let tx = tx.clone();
                            let steam = steam.clone();
                            tokio::spawn(async move {
                                let result = steam.achievements_with_global(&id).await;
                                let _ = tx.send(AppEvent::Achievements { id, result });
                            });
                        }
                        Effect::None => {}
                    }
                }
                AppEvent::Achievements { id, result } => {
                    state.achievements_cache.insert(id.clone(), result);
                    if let Screen::Detail(ref current_id) = state.screen
                        && *current_id == id
                    {
                        state.loading_achievements = false;
                    }
                }
            }
        }
    }

    let _ = disable_raw_mode();
    let _ = std::io::stdout().execute(LeaveAlternateScreen);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    fn make_game(appid: u32, name: &str, rtime_last_played: u64) -> Game {
        Game {
            id: GameId::Steam(appid),
            platform: Platform::Steam,
            name: name.to_string(),
            playtime_forever: Some(0),
            img_icon_url: None,
            rtime_last_played,
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
            grade: None,
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

    #[test]
    fn test_build_game_list_line_date_not_truncated() {
        let game_name = "Cyberpunk 2077";
        let date_str = "2026-07-30";
        let inner_width: u16 = 98;

        let line = build_game_list_line(game_name, date_str, inner_width);
        let total_len: usize = line.spans.iter().map(|s| s.content.len()).sum();

        assert_eq!(
            total_len, inner_width as usize,
            "Line width must equal inner_width"
        );
        assert_eq!(
            line.spans.last().unwrap().content,
            date_str,
            "Date must not be truncated"
        );
    }

    #[test]
    fn test_build_game_list_line_no_padding_when_full() {
        let game_name = "Cyberpunk 2077";
        let date_str = "2026-07-30";
        let inner_width: u16 = 24;

        let line = build_game_list_line(game_name, date_str, inner_width);
        let total_len: usize = line.spans.iter().map(|s| s.content.len()).sum();

        assert_eq!(total_len, 24);
        assert_eq!(line.spans[1].content, "");
    }

    #[test]
    fn test_build_game_list_line_with_unicode_symbols() {
        let game_name = "The Witcher 3: Wild Hunt™";
        let date_str = "2026-07-30";
        let inner_width: u16 = 60;

        let line = build_game_list_line(game_name, date_str, inner_width);
        assert_eq!(line.spans.last().unwrap().content, date_str);
    }

    #[test]
    fn test_state_new_tiebreak_on_name() {
        let g1 = make_game(1, "Zebra", 10);
        let g2 = make_game(2, "Alpha", 10);
        let state = State::new(vec![g1, g2]);
        assert_eq!(state.games[0].name, "Alpha");
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
        assert!(state.visible_achievements(&GameId::Steam(1)).is_none());
    }

    #[test]
    fn test_visible_achievements_none_when_cached_error() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state
            .achievements_cache
            .insert(GameId::Steam(1), Err(PlatformError::PrivateProfile));
        assert!(state.visible_achievements(&GameId::Steam(1)).is_none());
    }

    #[test]
    fn test_visible_achievements_view_modes() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state
            .achievements_cache
            .insert(GameId::Steam(1), Ok(make_achievement_set()));

        state.view_mode = AchievementViewMode::All;
        assert_eq!(
            state.visible_achievements(&GameId::Steam(1)).unwrap().len(),
            3
        );

        state.view_mode = AchievementViewMode::Remaining;
        let remaining = state.visible_achievements(&GameId::Steam(1)).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].apiname, "b_beta");

        state.view_mode = AchievementViewMode::Unlocked;
        let unlocked = state.visible_achievements(&GameId::Steam(1)).unwrap();
        assert_eq!(unlocked.len(), 2);
    }

    #[test]
    fn test_visible_achievements_sort_name() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state
            .achievements_cache
            .insert(GameId::Steam(1), Ok(make_achievement_set()));
        state.sort_mode = AchievementSortMode::Name;

        state.sort_dir = SortDirection::Ascending;
        let asc = state.visible_achievements(&GameId::Steam(1)).unwrap();
        assert_eq!(asc[0].name, "Alpha");
        assert_eq!(asc[2].name, "Gamma");

        state.sort_dir = SortDirection::Descending;
        let desc = state.visible_achievements(&GameId::Steam(1)).unwrap();
        assert_eq!(desc[0].name, "Gamma");
        assert_eq!(desc[2].name, "Alpha");
    }

    #[test]
    fn test_visible_achievements_sort_unlock_date() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state
            .achievements_cache
            .insert(GameId::Steam(1), Ok(make_achievement_set()));
        state.sort_mode = AchievementSortMode::UnlockDate;
        state.sort_dir = SortDirection::Ascending;

        let asc = state.visible_achievements(&GameId::Steam(1)).unwrap();
        assert_eq!(asc[0].name, "Beta");
        assert_eq!(asc[1].name, "Gamma");
        assert_eq!(asc[2].name, "Alpha");
    }

    #[test]
    fn test_visible_achievements_sort_global_percent() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state
            .achievements_cache
            .insert(GameId::Steam(1), Ok(make_achievement_set()));
        state.sort_mode = AchievementSortMode::GlobalPercent;
        state.sort_dir = SortDirection::Ascending;

        let asc = state.visible_achievements(&GameId::Steam(1)).unwrap();
        assert_eq!(asc[0].name, "Beta");
        assert_eq!(asc[1].name, "Alpha");
        assert_eq!(asc[2].name, "Gamma");
    }

    #[test]
    fn test_clamp_detail_selection() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state
            .achievements_cache
            .insert(GameId::Steam(1), Ok(make_achievement_set()));

        state.detail_selection_idx = 1;
        state.clamp_detail_selection(&GameId::Steam(1));
        assert_eq!(state.detail_selection_idx, 1);

        state.detail_selection_idx = 99;
        state.clamp_detail_selection(&GameId::Steam(1));
        assert_eq!(state.detail_selection_idx, 2);
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
        assert_eq!(effect, Effect::FetchAchievements(GameId::Steam(42)));
        assert_eq!(state.screen, Screen::Detail(GameId::Steam(42)));
        assert!(state.loading_achievements);
    }

    #[test]
    fn test_handle_list_key_enter_cache_hit_no_fetch() {
        let mut state = State::new(vec![make_game(42, "Game", 0)]);
        state
            .achievements_cache
            .insert(GameId::Steam(42), Ok(make_achievement_set()));

        let effect = state.handle_key(key(KeyCode::Enter));
        assert_eq!(effect, Effect::None);
        assert_eq!(state.screen, Screen::Detail(GameId::Steam(42)));
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
        assert_eq!(state.selection_idx, 0);

        state.handle_key(key(KeyCode::Down));
        assert_eq!(state.selection_idx, 1);

        state.handle_key(key(KeyCode::PageDown));
        assert_eq!(state.selection_idx, 2);

        state.handle_key(key(KeyCode::PageUp));
        assert_eq!(state.selection_idx, 0);
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
        state.screen = Screen::Detail(GameId::Steam(1));
        let effect = state.handle_key(key_with(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert_eq!(effect, Effect::Quit);
    }

    #[test]
    fn test_handle_detail_key_back_to_list() {
        for code in [KeyCode::Esc, KeyCode::Char('q'), KeyCode::Backspace] {
            let mut state = State::new(vec![make_game(1, "Game", 0)]);
            state.screen = Screen::Detail(GameId::Steam(1));
            state.handle_key(key(code));
            assert_eq!(state.screen, Screen::List);
        }
    }

    #[test]
    fn test_handle_detail_key_view_modes() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state.screen = Screen::Detail(GameId::Steam(1));
        state
            .achievements_cache
            .insert(GameId::Steam(1), Ok(make_achievement_set()));

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
            state.screen = Screen::Detail(GameId::Steam(1));
            state.sort_mode = if mode == AchievementSortMode::UnlockDate {
                AchievementSortMode::Name
            } else {
                AchievementSortMode::UnlockDate
            };
            state.sort_dir = SortDirection::Ascending;

            state.handle_key(key(code));
            assert_eq!(state.sort_mode, mode);
            assert_eq!(state.sort_dir, first_dir);

            state.handle_key(key(code));
            assert_eq!(state.sort_mode, mode);
            assert_ne!(state.sort_dir, first_dir);
        }
    }

    #[test]
    fn test_handle_detail_key_navigation() {
        let mut state = State::new(vec![make_game(1, "Game", 0)]);
        state.screen = Screen::Detail(GameId::Steam(1));
        state
            .achievements_cache
            .insert(GameId::Steam(1), Ok(make_achievement_set()));

        state.handle_key(key(KeyCode::Down));
        assert_eq!(state.detail_selection_idx, 1);

        state.handle_key(key(KeyCode::Up));
        assert_eq!(state.detail_selection_idx, 0);

        state.handle_key(key(KeyCode::PageDown));
        assert_eq!(state.detail_selection_idx, 2);

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
        state.screen = Screen::Detail(GameId::Steam(1));
        state.loading_achievements = true;
        let mut terminal = Terminal::new(TestBackend::new(60, 10)).unwrap();
        terminal.draw(|f| render(&state, f)).unwrap();
        let content = terminal.backend().to_string();
        assert!(content.contains("Loading"));
    }

    #[test]
    fn test_render_detail_cache_miss_renders_nothing_extra() {
        let mut state = State::new(vec![make_game(1, "Portal", 0)]);
        state.screen = Screen::Detail(GameId::Steam(1));
        let mut terminal = Terminal::new(TestBackend::new(60, 10)).unwrap();
        terminal.draw(|f| render(&state, f)).unwrap();
    }

    #[test]
    fn test_render_detail_no_stats() {
        let mut state = State::new(vec![make_game(1, "Portal", 0)]);
        state.screen = Screen::Detail(GameId::Steam(1));
        state.achievements_cache.insert(
            GameId::Steam(1),
            Err(PlatformError::NoStats {
                id: GameId::Steam(1),
            }),
        );
        let mut terminal = Terminal::new(TestBackend::new(60, 10)).unwrap();
        terminal.draw(|f| render(&state, f)).unwrap();
        let content = terminal.backend().to_string();
        assert!(content.contains("no achievements"));
    }

    #[test]
    fn test_render_detail_generic_error() {
        let mut state = State::new(vec![make_game(1, "Portal", 0)]);
        state.screen = Screen::Detail(GameId::Steam(1));
        state
            .achievements_cache
            .insert(GameId::Steam(1), Err(PlatformError::PrivateProfile));
        let mut terminal = Terminal::new(TestBackend::new(60, 10)).unwrap();
        terminal.draw(|f| render(&state, f)).unwrap();
        let content = terminal.backend().to_string();
        assert!(content.contains("private"));
    }

    #[test]
    fn test_render_detail_ok_set() {
        let mut state = State::new(vec![make_game(1, "Portal", 0)]);
        state.screen = Screen::Detail(GameId::Steam(1));
        state
            .achievements_cache
            .insert(GameId::Steam(1), Ok(make_achievement_set()));
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
            state.screen = Screen::Detail(GameId::Steam(1));
            state.sort_mode = mode;
            let mut terminal = Terminal::new(TestBackend::new(120, 10)).unwrap();
            terminal.draw(|f| render(&state, f)).unwrap();
            let content = terminal.backend().to_string();
            assert!(content.contains(sort_label));
        }
    }
}

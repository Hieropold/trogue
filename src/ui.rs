use chrono::{TimeZone, Utc};

use crate::game_library::{Achievement, Game, GameId, Platform, TrophyGrade};

// Prints the application title to the console.
//
// <purpose-start>
// This function is responsible for printing the application title to the console.
// <purpose-end>
//
// <inputs-start>
// - None.
// <inputs-end>
//
// <outputs-start>
// - None.
// <outputs-end>
//
// <side-effects-start>
// - **Prints to the console**: The application title is printed to the standard output.
// <side-effects-end>
pub fn print_title() {
    let title = r#"                                                                                                                                       
  ****           *                                                               
 *  *************                                                                
*     *********                                                                  
*     *  *                                                                       
 **  *  **         ***  ****       ****                  **   ****               
    *  ***          **** **** *   * ***  *     ****       **    ***  *    ***    
   **   **           **   ****   *   ****     *  ***  *   **     ****    * ***   
   **   **           **         **    **     *    ****    **      **    *   ***  
   **   **           **         **    **    **     **     **      **   **    *** 
   **   **           **         **    **    **     **     **      **   ********  
    **  **           **         **    **    **     **     **      **   *******   
     ** *      *     **         **    **    **     **     **      **   **        
      ***     *      ***         ******     **     **      ******* **  ****    * 
       *******        ***         ****       ********       *****   **  *******  
         ***                                   *** ***                   *****   
                                                    ***                          
                                              ****   ***                         
                                            *******  **                          
                                           *     ****                            
"#;

    println!("{title}");
}

// Prints the game title to the console.
//
// <purpose-start>
// This function is responsible for printing the game title to the console.
// <purpose-end>
//
// <inputs-start>
// - `game`: The `Game` struct to print the title of.
// <inputs-end>
//
// <outputs-start>
// - None.
// <outputs-end>
//
// <side-effects-start>
// - **Prints to the console**: The game title is printed to the standard output.
// <side-effects-end>
pub fn print_game_title(game: &Game) {
    println!("{}", game.name);
}

// Prints the game ID to the console.
//
// <purpose-start>
// This function is responsible for printing the game ID to the console.
// <purpose-end>
//
// <inputs-start>
// - `game`: The `Game` struct to print the ID of.
// <inputs-end>
//
// <outputs-start>
// - None.
// <outputs-end>
//
// <side-effects-start>
// - **Prints to the console**: The game ID is printed to the standard output.
// <side-effects-end>
pub fn print_game_id(game: &Game) {
    println!("{}", game.id);
}

// A wrapper around the `Game` struct to provide display formatting.
pub struct DisplayableGame {
    pub game: Game,
}

impl DisplayableGame {
    // Formats the game information according to a pattern.
    //
    // <purpose-start>
    // This function formats the game information into a string based on a provided pattern.
    // The pattern can contain tokens that are replaced with game data.
    // <purpose-end>
    //
    // <inputs-start>
    // - `pattern`: A string containing the format pattern.
    // <inputs-end>
    //
    // <outputs-start>
    // - `String`: The formatted string.
    // <outputs-end>
    //
    // <side-effects-start>
    // - None.
    // <side-effects-end>
    pub fn format(&self, pattern: &str) -> String {
        let mut result = String::new();

        for ch in pattern.chars() {
            match ch {
                'n' => result.push_str(&self.game.name),
                'i' => result.push_str(&self.game.id.to_string()),
                'p' => result.push_str(&self.game.platform.to_string()),
                _ => result.push(ch),
            }
        }

        result
    }
}

// A wrapper around the `Achievement` struct to provide display formatting.
pub struct DisplayableAchievement {
    pub achievement: Achievement,
}

impl DisplayableAchievement {
    // Formats the achievement information according to a pattern.
    //
    // <purpose-start>
    // This function formats the achievement information into a string based on a provided pattern.
    // The pattern can contain tokens that are replaced with achievement data.
    // <purpose-end>
    //
    // <inputs-start>
    // - `pattern`: A string containing the format pattern.
    // <inputs-end>
    //
    // <outputs-start>
    // - `String`: The formatted string.
    // <outputs-end>
    //
    // <side-effects-start>
    // - None.
    // <side-effects-end>
    pub fn format(&self, pattern: &str) -> String {
        let mut result = String::new();

        for ch in pattern.chars() {
            match ch {
                'i' => result.push_str(&self.achievement.apiname),
                'n' => result.push_str(&self.achievement.name),
                'd' => result.push_str(&self.achievement.description),
                's' => result.push_str(if self.achievement.achieved > 0 {
                    "Y"
                } else {
                    "N"
                }),
                't' => result.push_str(&self.formatted_unlocktime()),
                'g' => result.push_str(
                    &self
                        .achievement
                        .grade
                        .map_or(String::new(), |g| g.to_string()),
                ),
                _ => result.push(ch),
            }
        }

        result
    }

    // Renders a card-like representation of the achievement.
    //
    // <purpose-start>
    // This function creates a string that represents the achievement in a card-like format.
    // <purpose-end>
    //
    // <inputs-start>
    // - None.
    // <inputs-end>
    //
    // <outputs-start>
    // - `String`: The card-like representation of the achievement.
    // <outputs-end>
    //
    // <side-effects-start>
    // - None.
    // <side-effects-end>
    pub fn render_card(&self) -> String {
        let mut card = String::new();
        let achieved = if self.achievement.achieved == 1 {
            "Y"
        } else {
            "N"
        };
        let unlock_date = self.formatted_unlocktime();

        let apiname_length = self.achievement.apiname.len();
        let unlock_length = unlock_date.len();

        let longest_length = if apiname_length > unlock_length {
            apiname_length
        } else {
            unlock_length
        };

        // Generate top ┌──────┐
        card.push('┌');
        let horizontal_line_width = longest_length + 8;
        for _ in 0..horizontal_line_width {
            card.push('─');
        }
        card.push_str("┐\n");

        card.push_str(&format!(
            "│ Name: {:>longest_length$} │\n",
            self.achievement.apiname
        ));

        let achieved_width = longest_length - 4;
        card.push_str(&format!(
            "│ Achieved: {:>achieved_width$} │\n",
            achieved,
            achieved_width = achieved_width
        ));

        card.push_str(&format!(
            "│ Date: {:>longest_length$} │\n",
            self.formatted_unlocktime()
        ));

        // Lower └─────────┘
        card.push('└');
        for _i in 0..horizontal_line_width {
            card.push('─');
        }
        card.push_str("┘\n");

        card
    }

    // Formats the unlock time into a human-readable string.
    //
    // <purpose-start>
    // This function converts the Unix timestamp of the achievement's unlock time into a formatted string safely.
    // <purpose-end>
    //
    // <inputs-start>
    // - None.
    // <inputs-end>
    //
    // <outputs-start>
    // - `String`: The formatted unlock time.
    // <outputs-end>
    //
    // <side-effects-start>
    // - None.
    // <side-effects-end>
    fn formatted_unlocktime(&self) -> String {
        let ts: i64 = self.achievement.unlocktime.try_into().unwrap_or(0);
        let datetime = Utc
            .timestamp_opt(ts, 0)
            .single()
            .unwrap_or_else(|| Utc.timestamp_opt(0, 0).unwrap());

        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

pub enum ViewData {
    ListGames(Vec<Game>, Option<String>, String), // games, filter, pattern
    None,
}

pub struct Renderer;

impl Renderer {
    pub fn render(
        view_data: ViewData,
        writer: &mut (dyn std::io::Write + Send),
    ) -> std::io::Result<()> {
        match view_data {
            ViewData::ListGames(games, filter, pattern) => {
                match filter {
                    Some(f) => {
                        writeln!(writer, "Displaying games filtered by: {}", f)?;
                    }
                    None => {
                        writeln!(writer, "Displaying all games:")?;
                    }
                }
                for game in games {
                    let displayable = DisplayableGame { game };
                    writeln!(writer, "{}", displayable.format(&pattern))?;
                }
            }
            ViewData::None => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_game() -> Game {
        Game {
            id: GameId::Steam(123),
            platform: Platform::Steam,
            name: "Test Game".to_string(),
            playtime_forever: Some(100),
            img_icon_url: Some("icon_url".to_string()),
            rtime_last_played: 0,
        }
    }

    fn create_mock_achievement(achieved: u8, unlocktime: u64) -> Achievement {
        Achievement {
            apiname: "test_api".to_string(),
            name: "Test Achievement".to_string(),
            description: "Test Description".to_string(),
            achieved,
            unlocktime,
            global_percent: None,
            grade: Some(TrophyGrade::Gold),
        }
    }

    #[test]
    fn test_displayable_game_format() {
        let game = create_mock_game();
        let displayable_game = DisplayableGame { game };

        let formatted = displayable_game.format("n (i) [p]");
        assert_eq!(formatted, "Test Game (steam:123) [steam]");
    }

    #[test]
    fn test_displayable_achievement_format_achieved() {
        let achievement = create_mock_achievement(1, 1672531200); // 2023-01-01 00:00:00
        let displayable_achievement = DisplayableAchievement { achievement };

        let formatted = displayable_achievement.format("i: n - s, t, d [g]");
        assert_eq!(
            formatted,
            "test_api: Test Achievement - Y, 2023-01-01 00:00:00, Test Description [Gold]"
        );
    }

    #[test]
    fn test_displayable_achievement_format_not_achieved() {
        let achievement = create_mock_achievement(0, 0);
        let displayable_achievement = DisplayableAchievement { achievement };

        let formatted = displayable_achievement.format("i: n - s, t, d");
        assert_eq!(
            formatted,
            "test_api: Test Achievement - N, 1970-01-01 00:00:00, Test Description"
        );
    }

    #[test]
    fn test_formatted_unlocktime() {
        let achievement = create_mock_achievement(1, 1672531200); // 2023-01-01 00:00:00
        let displayable_achievement = DisplayableAchievement { achievement };

        let formatted_time = displayable_achievement.formatted_unlocktime();
        assert_eq!(formatted_time, "2023-01-01 00:00:00");
    }

    #[test]
    fn test_render_card_achieved() {
        let achievement = create_mock_achievement(1, 1672531200); // 2023-01-01 00:00:00
        let displayable_achievement = DisplayableAchievement { achievement };

        let card = displayable_achievement.render_card();
        let expected_card = "┌───────────────────────────┐\n│ Name:            test_api │\n│ Achieved:               Y │\n│ Date: 2023-01-01 00:00:00 │\n└───────────────────────────┘\n";
        assert_eq!(card, expected_card);
    }

    #[test]
    fn test_render_card_not_achieved() {
        let achievement = create_mock_achievement(0, 0);
        let displayable_achievement = DisplayableAchievement { achievement };

        let card = displayable_achievement.render_card();
        let expected_card = "┌───────────────────────────┐\n│ Name:            test_api │\n│ Achieved:               N │\n│ Date: 1970-01-01 00:00:00 │\n└───────────────────────────┘\n";
        assert_eq!(card, expected_card);
    }

    #[test]
    fn test_renderer_render_list_games_with_filter() {
        let mut buf = Vec::new();
        let view_data = ViewData::ListGames(
            vec![create_mock_game()],
            Some("test".to_string()),
            "n (i)".to_string(),
        );
        Renderer::render(view_data, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Displaying games filtered by: test"));
        assert!(output.contains("Test Game (steam:123)"));
    }

    #[test]
    fn test_renderer_render_list_games_without_filter() {
        let mut buf = Vec::new();
        let view_data = ViewData::ListGames(vec![create_mock_game()], None, "n (i)".to_string());
        Renderer::render(view_data, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Displaying all games:"));
        assert!(output.contains("Test Game (steam:123)"));
    }

    #[test]
    fn test_renderer_render_none() {
        let mut buf = Vec::new();
        Renderer::render(ViewData::None, &mut buf).unwrap();
        assert!(buf.is_empty());
    }

    #[test]
    fn test_print_title_does_not_panic() {
        print_title();
    }

    #[test]
    fn test_print_game_title_does_not_panic() {
        print_game_title(&create_mock_game());
    }

    #[test]
    fn test_print_game_id_does_not_panic() {
        print_game_id(&create_mock_game());
    }
}

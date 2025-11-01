use chrono::{TimeZone, Utc};

use crate::steam_api::{Achievement, Game};

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
    println!("{}", game.appid);
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
                'i' => result.push_str(&self.game.appid.to_string()),
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
                's' => result.push_str(if self.achievement.achieved > 0 { "Y" } else { "N" }),
                't' => result.push_str(&self.formatted_unlocktime()),
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
        let achieved = if self.achievement.achieved == 1 { "Y" } else { "N" };
        let unlock_date = self.formatted_unlocktime();

        let apiname_length = self.achievement.apiname.len();
        let unlock_length = unlock_date.len();

        let longest_length = if apiname_length > unlock_length {
            apiname_length
        } else {
            unlock_length
        };

        // Generate top ┌──────┐
        card.push_str("┌");
        let horizontal_line_width = longest_length + 8;
        for _ in 0..horizontal_line_width {
            card.push_str("─");
        }
        card.push_str("┐\n");

        card.push_str(&format!("│ Name: {:>longest_length$} │\n", self.achievement.apiname));

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
        card.push_str("└");
        for _i in 0..horizontal_line_width {
            card.push_str("─");
        }
        card.push_str("┘\n");

        card
    }

    // Formats the unlock time into a human-readable string.
    //
    // <purpose-start>
    // This function converts the Unix timestamp of the achievement's unlock time into a formatted string.
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
        let ts = self.achievement.unlocktime.try_into().unwrap();
        let datetime = Utc
            .timestamp_opt(ts, 0)
            .single()
            .expect("Invalid Unix timestamp");

        // Format the NaiveDateTime into a human-readable string
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_game() -> Game {
        Game {
            appid: 123,
            name: "Test Game".to_string(),
            playtime_forever: 100,
            img_icon_url: "icon_url".to_string(),
            playtime_windows_forever: 100,
            playtime_mac_forever: 0,
            playtime_linux_forever: 0,
            rtime_last_played: 0,
            playtime_disconnected: 0,
        }
    }

    fn create_mock_achievement(achieved: u8, unlocktime: u64) -> Achievement {
        Achievement {
            apiname: "test_api".to_string(),
            name: "Test Achievement".to_string(),
            description: "Test Description".to_string(),
            achieved,
            unlocktime,
        }
    }

    #[test]
    fn test_displayable_game_format() {
        let game = create_mock_game();
        let displayable_game = DisplayableGame { game };

        let formatted = displayable_game.format("n (i)");
        assert_eq!(formatted, "Test Game (123)");
    }

    #[test]
    fn test_displayable_achievement_format_achieved() {
        let achievement = create_mock_achievement(1, 1672531200); // 2023-01-01 00:00:00
        let displayable_achievement = DisplayableAchievement { achievement };

        let formatted = displayable_achievement.format("i: n - s, t, d");
        assert_eq!(
            formatted,
            "test_api: Test Achievement - Y, 2023-01-01 00:00:00, Test Description"
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
}
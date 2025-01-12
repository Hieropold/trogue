use chrono::{TimeZone, Utc};

use crate::steam_api::{Achievement, Game};

pub fn print_title() {
    let title = r#"                                                                                                                                       
  ****           *                                      *                                                                          
  *  *************                                     **                                                                           
 *     *********                                       **                                                                           
 *     *  *                                            **                                                                           
  **  *  **         ***  ****       ****       ****    **        **   ****      ***  ****       ****       ****                     
     *  ***          **** **** *   * ***  *   * ***  * **  ***    **    ***  *   **** **** *   * ***  *   * ***  * *** **** ****    
    **   **           **   ****   *   ****   *   ****  ** * ***   **     ****     **   ****   *   ****   *   ****   *** **** ***  * 
    **   **           **         **    **   **    **   ***   ***  **      **      **         **    **   **    **     **  **** ****  
    **   **           **         **    **   **    **   **     **  **      **      **         **    **   **    **     **   **   **   
    **   **           **         **    **   **    **   **     **  **      **      **         **    **   **    **     **   **   **   
     **  **           **         **    **   **    **   **     **  **      **      **         **    **   **    **     **   **   **   
      ** *      *     **         **    **   **    **   **     **  **      **      **         **    **   **    **     **   **   **   
       ***     *      ***         ******    *******    **     **   *********      ***         ******     ******      **   **   **   
        *******        ***         ****     ******     **     **     **** ***      ***         ****       ****       ***  ***  ***  
          ***                               **          **    **           ***                                        ***  ***  *** 
                                            **                *     *****   ***                                                     
                                            **               *    ********  **                                                      
                                             **             *    *      ****                                                        
                                                           *                                                                        
"#;

    println!("{title}");
}

pub fn print_game_title(game: &Game) {
    println!("{}", game.name);
}

pub fn print_game_id(game: &Game) {
    println!("{}", game.appid);
}

pub struct DisplayableGame {
    pub game: Game,
}

impl DisplayableGame {
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

pub struct DisplayableAchievement {
    pub achievement: Achievement,
}

impl DisplayableAchievement {
    pub fn format(&self, pattern: &str) -> String {
        let mut result = String::new();

        for ch in pattern.chars() {
            match ch {
                'n' => result.push_str(&self.achievement.apiname),
                's' => result.push_str(if self.achievement.achieved > 0 { "Y" } else { "N" }),
                't' => result.push_str(&self.formatted_unlocktime()),
                _ => result.push(ch),
            }
        }

        result
    }

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

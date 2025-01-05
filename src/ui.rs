use crate::steam_api::Game;

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
  pub game: Game
}

impl DisplayableGame {
  pub fn format(&self, pattern: &str) -> String {
    let mut result = String::new();

    for ch in pattern.chars() {
      match ch {
        'n' => result.push_str(&self.game.name),
        'i' => result.push_str(&self.game.appid.to_string()),
        _ => result.push(ch)
      }
    }

    result
  }
}
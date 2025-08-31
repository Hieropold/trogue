/// Allows the user to select a game from a list.
///
/// <purpose-start>
/// This function provides a text-based user interface for selecting a game from a list.
/// It allows the user to filter the list by typing a search query.
/// <purpose-end>
///
/// <inputs-start>
/// - `games`: A vector of `steam_api::Game` structs to select from.
/// <inputs-end>
///
/// <outputs-start>
/// - None.
/// <outputs-end>
///
/// <side-effects-start>
/// - **Enters raw mode**: The terminal is put into raw mode to handle key events.
/// - **Clears the screen**: The terminal screen is cleared.
/// - **Prints to the console**: The list of games is printed to the console.
/// <side-effects-end>
///
/// # Note
/// This function is currently not used in the application.
fn select_game(games: &Vec<steam_api::Game>) {
    // let mut idx = 0;
    // for game in games {
    //     idx += 1;
    //     println!("[{}] {}", idx, game.name);
    // }

    let mut name_filter = String::new();

    // Initialize term to enter raw mode
    terminal::enable_raw_mode().expect("Failed to enable terminal raw mode");

    // Clear terminal screen
    execute!(
        stdout(),
        cursor::MoveTo(0, 0),
        terminal::Clear(terminal::ClearType::All)
    )
    .unwrap();

    loop {
        // io::stdout().flush().map_err(|e| e.to_string())?;

        // io::stdin().read_line(&mut name_filter).map_err(|e| e.to_string())?;

        // io::stdin().read_to_string(name_filter).map_err(|e| e.to_string());

        // name_filter = name_filter.trim().to_string();

        /*let mut filtered_games = games.iter().filter(|game| {
            if name_filter.len() == 0 {
                return true;
            }
            return game.name.to_lowercase().contains(&name_filter.to_lowercase());
        }).collect::<Vec<&steam_api::Game>>();

        if filtered_games.len() == 0 {
            println!("No games found.");
            continue;
        }*/

        // Read the next event from the terminal
        if let Event::Key(key_event) = crossterm::event::read().expect("Failed to read key event") {
            match key_event.code {
                KeyCode::Char(c) => {
                    // Append the character to the filter
                    name_filter.push(c);
                }
                KeyCode::Backspace => {
                    // Remove the last character from the filter
                    name_filter.pop();
                }
                KeyCode::Esc | KeyCode::Enter => {
                    break;
                }
                _ => {}
            }
        }

        execute!(
            stdout(),
            cursor::MoveTo(0, 0),
            terminal::Clear(terminal::ClearType::All)
        )
        .unwrap();
        print!("{}\n", name_filter);

        // Filter the games based on the current filter input
        let mut filtered_games = games.clone();
        filtered_games.retain(|entry| {
            entry
                .name
                .to_lowercase()
                .contains(&name_filter.to_lowercase())
        });

        // Print out the filtered list
        let mut idx = 0;
        for game in filtered_games {
            idx += 1;
            execute!(stdout(), cursor::MoveTo(0, idx)).unwrap();
            println!("{}", game.name);
        }

        // Move the cursor to end of first line
        let name_length: u16 = name_filter
            .len()
            .try_into()
            .expect("Name length too long to fit into u16");
        execute!(stdout(), cursor::MoveTo(name_length, 0)).unwrap();
    }

    // Reset terminal mode
    terminal::disable_raw_mode().expect("Failed to disable the raw mode");
}

/*print!("Please select game [1 - {}]: ", filtered_games.len());
io::stdout().flush().map_err(|e| e.to_string())?;

let game = games.get(entered_idx as usize - 1).ok_or("Invalid game index.")?;

return Ok(game);*/

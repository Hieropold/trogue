# Trophyroom

Command line tool to view the Steam achievements.

# Configure

Configuration is done through environment:
* `TROPHYROOM_STEAM_API_KEY`
* `TROPHYROOM_STEAM_ID`

# Usage

Some possible usage examples:
* `trophyroom -l` will list all games in the library, outputting game names only
* `trophyroom -l redemption -p 'i - n'` will list games containing "dragon" in the name, and output game id and game name separated by hyphen
* `trophyroom -a 48700` will display achievements for a specific game

Run `trophyroom -h` for a full list of available options.

# Build

To build the tool:
```
cargo build
cargo build --release
```

or

```
cargo run
```

To format code:
```
cargo fmt
```

To run linting:
```
cargo clippy
```

# Development

## Steam
https://developer.valvesoftware.com/wiki/Steam_Web_API

## PSN
https://andshrew.github.io/PlayStation-Trophies/#/APIv2
* Log in on https://www.playstation.com/sr-rs/
* Get token on https://ca.account.sony.com/api/v1/ssocookie
* Use it in requests

# Roadmap

- [x] Achievement card - name, status, date, progress
- [x] CLI mode additionally to interactive mode
- [ ] Game name search with typeahead in interactive mode
- [ ] Game name tab completion in CLI mode
- [ ] Add support for PSN
- [ ] Add support for Xbox

# Alternative names
* trphlib - trophylib
* atk - achievement toolkit
* atlkt - achievement toolkit
* gmslb - games library
* gamelib - games library
# Trophyroom

Command line tool to view the Steam achievements.

# Configure

Configuration is done through environment:
* `TROPHYROOM_STEAM_API_KEY`
* `TROPHYROOM_STEAM_ID`

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

## PSN
https://andshrew.github.io/PlayStation-Trophies/#/APIv2
* Log in on https://www.playstation.com/sr-rs/
* Get token on https://ca.account.sony.com/api/v1/ssocookie
* Use it in requests

# Roadmap

- [ ] Achievement card - name, status, date, progress
- [ ] CLI mode additionally to interactive mode
- [ ] Game name search with typeahead in interactive mode
- [ ] Game name tab completion in CLI mode
- [ ] Add support for PSN
- [ ] Add support for Xbox
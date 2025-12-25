```
  /###           /                                                  
 /  ############/                                                   
/     #########                                                     
#     /  #                                                          
 ##  /  ##                                                          
    /  ###     ###  /###     /###     /###    ##   ####      /##    
   ##   ##      ###/ #### / / ###  / /  ###  / ##    ###  / / ###   
   ##   ##       ##   ###/ /   ###/ /    ###/  ##     ###/ /   ###  
   ##   ##       ##       ##    ## ##     ##   ##      ## ##    ### 
   ##   ##       ##       ##    ## ##     ##   ##      ## ########  
    ##  ##       ##       ##    ## ##     ##   ##      ## #######   
     ## #      / ##       ##    ## ##     ##   ##      ## ##        
      ###     /  ##       ##    ## ##     ##   ##      /# ####    / 
       ######/   ###       ######   ########    ######/ ## ######/  
         ###      ###       ####      ### ###    #####   ## #####   
                                           ###                      
                                     ####   ###                     
                                   /######  /#                      
                                  /     ###/                
```

# TROphy cataloGUE

Command line tool to view the Steam achievements.

# Install
```
sudo add-apt-repository ppa:hieropold/ppa
sudo apt update
sudo apt install trogue
```

# Configure

Configuration is done through environment:
* `TROGUE_STEAM_API_KEY`
* `TROGUE_STEAM_ID`

# Usage

Some possible usage examples:
* `trogue -l` will list all games in the library, outputting game names only
* `trogue -l redemption -p 'i - n'` will list games containing "redemption" in the name, and output game id and game name separated by hyphen
* `trogue -a 48700` will display achievements for a specific game

Run `trogue -h` for a full list of available options.

# Shell Completion

Trogue supports shell completion for bash and zsh. This enables tab completion for commands and their options.

## Installation

### Bash
```bash
# Generate completion script and append to ~/.bashrc
echo '# trogue completion' >> ~/.bashrc
trogue completions bash >> ~/.bashrc

# Reload bash completion
source ~/.bashrc
```

### Zsh
```bash
# Generate completion script
trogue completions zsh > ~/.zsh/completions/_trogue

# Add to ~/.zshrc if not already present:
# fpath=(~/.zsh/completions $fpath)

# Reload zsh completion
source ~/.zshrc
```

## Usage

After installation, you can use tab completion:

```bash
trogue <Tab>          # Shows: achievements completions dashboard list progress
trogue ac<Tab>        # Autocompletes to: trogue achievements
trogue list --<Tab>   # Shows available options: --filter --pattern --help
```

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

# Test

Run the tests:
```
cargo test
cargo test -- --nocapture
```

Run the tests with coverage.
```
cargo tarpaulin --out Html -- --test-threads=1
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
- [x] List achievements by game name filter, display all matching games if multiple
- [ ] Dashboard with all 100% games
- [ ] Latest game dashboard - achievements progress, list remaining
- [ ] Game name tab completion in CLI mode
- [ ] Game name search with typeahead in interactive mode
- [ ] Add support for PSN
- [ ] Add support for Xbox

# Packaging

To create a Debian package and upload it to a Launchpad PPA:

```bash
./build-and-upload-to-ppa.sh
```

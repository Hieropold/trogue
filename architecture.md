# Application Architecture

## Application Purpose

The `trophyroom` application is a command-line interface (CLI) tool designed to interact with the Steam API. It allows users to retrieve information about their owned games and achievement status.

## Features

- **List Games:** Display a list of all games owned by the user, with an option to filter by name.
- **List Achievements:** Show a list of all achievements for a specific game, with options to filter by achieved status and include global achievement percentages.
- **Show Progress:** Display the achievement progress for a specific game as a progress bar.
- **Dashboard:** Show a dashboard of the 10 most recently played games and their achievement progress.

## Architecture Overview

The application is structured into several modules, each with a specific responsibility:

- **`main`:** The entry point of the application. It handles command-line argument parsing and dispatches commands to the `app` module.
- **`cfg`:** Responsible for loading and managing the application's configuration, which includes the Steam API key and Steam ID.
- **`app`:** The core of the application. It contains the main application logic and orchestrates the interactions between the other modules.
- **`steam_api`:** A client for interacting with the Steam API. It handles the details of making HTTP requests and deserializing the responses.
- **`ui`:** Contains functions for displaying information to the user in a formatted way.
- **`tui`:** Contains a text-based user interface for selecting a game from a list (currently unused).

## Module Descriptions

### `main.rs`

The `main` module is responsible for:

- Parsing command-line arguments using the `clap` library.
- Loading the application configuration using the `cfg` module.
- Creating an instance of the `App` struct from the `app` module.
- Calling the appropriate method on the `App` instance based on the parsed command-line arguments.

### `cfg.rs`

The `cfg` module is responsible for:

- Defining the `Cfg` struct, which holds the application's configuration.
- Loading the configuration from environment variables (`TROPHYROOM_STEAM_API_KEY` and `TROPHYROOM_STEAM_ID`).

### `app.rs`

The `app` module is the core of the application and is responsible for:

- Initializing the `Api` client from the `steam_api` module.
- Handling the application's main logic, such as listing games, showing achievement progress, and displaying the dashboard.
- Calling the `steam_api` module to retrieve data from the Steam API.
- Calling the `ui` module to display the data to the user.

### `steam_api.rs`

The `steam_api` module is responsible for:

- Defining the data structures that represent the responses from the Steam API.
- Providing a client (`Api` struct) for interacting with the Steam API.
- Sending HTTP requests to the Steam API and deserializing the JSON responses.

### `ui.rs`

The `ui` module is responsible for:

- Providing functions for displaying formatted output to the user.
- Defining `DisplayableGame` and `DisplayableAchievement` structs that wrap the `Game` and `Achievement` structs from the `steam_api` module and provide formatting capabilities.

### `tui.rs`

The `tui` module contains a text-based user interface for selecting a game from a list. This module is currently unused.

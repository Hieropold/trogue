# Application Architecture

## Application Purpose

The `trogue` application is a command-line interface (CLI) tool designed to interact with the Steam API. It allows users to retrieve information about their owned games and achievement status.

## Features

- **List Games:** Display a list of all games owned by the user, with an option to filter by name.
- **List Achievements:** Show a list of all achievements for a specific game, with options to filter by achieved status and include global achievement percentages.
- **Show Progress:** Display the achievement progress for a specific game as a progress bar.
- **Dashboard:** Show a dashboard of the 10 most recently played games and their achievement progress.

## Architecture Overview

The application is designed around a **plugin-based architecture** to promote modularity, extensibility, and a clear separation of concerns. The core application provides a foundation for plugins to build upon, offering shared services like configuration management, Steam API access, and UI rendering utilities.

The main components of this architecture are:

- **`Core`:** The central part of the application, responsible for initializing shared services, loading plugins, and dispatching commands. It does not contain any feature-specific logic itself.
- **`Plugins`:** Self-contained modules that implement specific features (e.g., `list-games`, `dashboard`). Each plugin defines its own command-line interface, contains the logic to perform its task, and utilizes services provided by the core.
- **`Shared Services`:** A collection of modules that provide common functionality accessible to all plugins. This includes:
    - **`cfg`:** For application configuration.
    - **`steam_api`:** For interacting with the Steam API.
    - **`ui`:** For rendering formatted output to the console.

This architecture makes it easy to add new features by simply creating a new plugin, without modifying the application's core logic.

## Module Descriptions

### `main.rs`

The entry point of the application. Its responsibilities are:
- Initializing the `App` context, which holds shared services.
- Loading all available plugins from the `plugins` module.
- Dynamically constructing the main `clap` command-line parser by aggregating the command definitions from all loaded plugins.
- Parsing command-line arguments.
- Identifying the invoked command and dispatching it to the `execute` method of the corresponding plugin.

### `app.rs`

Defines the `App` struct, which acts as a shared context for all plugins. It is initialized in `main.rs` and passed to plugins when they are executed. It holds instances of shared services:
- `cfg::Cfg`: The application configuration.
- `steam_api::Api`: The client for the Steam API.

### `plugins/mod.rs`

The heart of the plugin system. It is responsible for:
- Defining the `Plugin` trait, which all plugins must implement. This trait standardizes how plugins define their commands and execute their logic.
- Registering and providing a list of all available plugins to the `main` module.

Each feature is implemented as a separate plugin module within the `src/plugins/` directory (e.g., `src/plugins/list_games.rs`, `src/plugins/dashboard.rs`).

### `cfg.rs`

Responsible for loading and managing the application's configuration, which includes the Steam API key and Steam ID from environment variables.

### `steam_api.rs`

Provides a client for interacting with the Steam API. It handles HTTP requests, deserializes responses, and defines the data structures for the API's data.

### `ui.rs`

A utility module that provides functions for displaying formatted output to the user. It can be used by any plugin to ensure a consistent look and feel across the application.

### `tui.rs`

Contains a text-based user interface for selecting a game from a list. This module is currently unused but could be integrated into a plugin in the future.
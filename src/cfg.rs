use std::env;

// Represents the application configuration.
//
// <purpose-start>
// This struct holds the configuration for the application, including the Steam API key and Steam ID.
// <purpose-end>
pub struct Cfg {
    api_key: String,
    steam_id: String,
}

impl Cfg {
    // Creates a new, empty `Cfg` instance.
    //
    // <purpose-start>
    // This function initializes an empty `Cfg` struct.
    // <purpose-end>
    //
    // <inputs-start>
    // - None.
    // <inputs-end>
    //
    // <outputs-start>
    // - `Self`: A new `Cfg` instance.
    // <outputs-end>
    //
    // <side-effects-start>
    // - None.
    // <side-effects-end>
    pub fn new() -> Self {
        Self {
            api_key: "".to_string(),
            steam_id: "".to_string(),
        }
    }

    // Returns the Steam API key.
    //
    // <purpose-start>
    // This function returns a reference to the Steam API key.
    // <purpose-end>
    //
    // <inputs-start>
    // - None.
    // <inputs-end>
    //
    // <outputs-start>
    // - `&str`: A reference to the Steam API key.
    // <outputs-end>
    //
    // <side-effects-start>
    // - None.
    // <side-effects-end>
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    // Returns the Steam ID.
    //
    // <purpose-start>
    // This function returns a reference to the Steam ID.
    // <purpose-end>
    //
    // <inputs-start>
    // - None.
    // <inputs-end>
    //
    // <outputs-start>
    // - `&str`: A reference to the Steam ID.
    // <outputs-end>
    //
    // <side-effects-start>
    // - None.
    // <side-effects-end>
    pub fn steam_id(&self) -> &str {
        &self.steam_id
    }

    // Loads the configuration from environment variables.
    //
    // <purpose-start>
    // This function loads the Steam API key and Steam ID from environment variables.
    // <purpose-end>
    //
    // <inputs-start>
    // - None.
    // <inputs-end>
    //
    // <outputs-start>
    // - `Ok(())` if the configuration was loaded successfully.
    // - `Err(&str)` if an environment variable is missing.
    // <outputs-end>
    //
    // <side-effects-start>
    // - **Reads environment variables**: Reads the `TROGUE_STEAM_API_KEY` and `TROGUE_STEAM_ID` environment variables.
    // <side-effects-end>
    pub fn load(&mut self) -> Result<(), &str> {
        match Cfg::read_env("TROGUE_STEAM_API_KEY") {
            Ok(api_key) => self.api_key = api_key,
            Err(_) => return Err("Missing TROGUE_STEAM_API_KEY environment variable."),
        }

        match Cfg::read_env("TROGUE_STEAM_ID") {
            Ok(steam_id) => self.steam_id = steam_id,
            Err(_) => return Err("Missing TROGUE_STEAM_ID environment variable."),
        }

        Ok(())
    }

    // Reads an environment variable.
    //
    // <purpose-start>
    // This function reads the value of an environment variable.
    // <purpose-end>
    //
    // <inputs-start>
    // - `key`: The name of the environment variable to read.
    // <inputs-end>
    //
    // <outputs-start>
    // - `Ok(String)` if the environment variable is found.
    // - `Err(env::VarError)` if the environment variable is not found.
    // <outputs-end>
    //
    // <side-effects-start>
    // - **Reads environment variables**: Reads the specified environment variable.
    // <side-effects-end>
    pub fn read_env(key: &str) -> Result<String, env::VarError> {
        env::var(key)
    }
}

impl Default for Cfg {
    // Creates a default `Cfg` instance.
    //
    // <purpose-start>
    // This function creates a default `Cfg` instance by calling `Cfg::new()`.
    // <purpose-end>
    //
    // <inputs-start>
    // - None.
    // <inputs-end>
    //
    // <outputs-start>
    // - `Self`: A new `Cfg` instance.
    // <outputs-end>
    //
    // <side-effects-start>
    // - None.
    // <side-effects-end>
    fn default() -> Self {
        Self::new()
    }
}

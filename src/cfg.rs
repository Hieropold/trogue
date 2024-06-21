use std::env;

pub struct Cfg {
    api_key: String,
    steam_id: String,
}

impl Cfg {
    pub fn new() -> Self {
        Self {
            api_key: "".to_string(),
            steam_id: "".to_string(),
        }
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn steam_id(&self) -> &str {
        &self.steam_id
    }

    pub fn load(&mut self) -> Result<(), &str> {
        match Cfg::read_env("TROPHYROOM_STEAM_API_KEY") {
            Ok(api_key) => self.api_key = api_key,
            Err(_) => return Err("Missing TROPHYROOM_STEAM_API_KEY environment variable."),
        }

        match Cfg::read_env("TROPHYROOM_STEAM_ID") {
            Ok(steam_id) => self.steam_id = steam_id,
            Err(_) => return Err("Missing TROPHYROOM_STEAM_ID environment variable."),
        }

        Ok(())
    }

    pub fn read_env(key: &str) -> Result<String, env::VarError> {
        env::var(key)
    }
}

impl Default for Cfg {
    fn default() -> Self {
        Self::new()
    }
}

use crate::{cfg::Cfg, steam_api::Api};

/// The main application structure.
///
/// <purpose-start>
/// This struct holds the state of the application, including the Steam API client.
/// <purpose-end>
pub struct AppContext {
    pub api: Api,
}

impl AppContext {
    /// Creates a new `AppContext` instance.
    ///
    /// <purpose-start>
    /// This function initializes the `AppContext` struct, creating a new `Api` instance with the provided configuration.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `cfg`: The application configuration, containing the API key and Steam ID.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - `AppContext`: A new `AppContext` instance.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - None.
    /// <side-effects-end>
    pub fn new(cfg: Cfg) -> AppContext {
        let api = Api::new(cfg.api_key().to_string(), cfg.steam_id().to_string());

        AppContext { api }
    }
}

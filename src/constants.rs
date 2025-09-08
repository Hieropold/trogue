/// The base URL for the Steam API.
///
/// <purpose-start>
/// This constant provides a single, authoritative source for the Steam API's base URL.
/// Using a constant ensures consistency and makes it easier to update the URL if it ever changes.
/// <purpose-end>
///
/// <inputs-start>
/// - None
/// <inputs-end>
///
/// <outputs-start>
/// - A string slice representing the base URL of the Steam API.
/// <outputs-end>
///
/// <side-effects-start>
/// - None
/// <side-effects-end>
pub const STEAM_API_BASE_URL: &str = "http://api.steampowered.com";

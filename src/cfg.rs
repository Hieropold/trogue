use std::env;

pub fn read_env(key: &str) -> String {
    env::var(key).expect(&format!("{} must be set", key))
}

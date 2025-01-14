use std::fs;

use crate::config::Config;

pub fn get_config() -> Result<Config, String> {
    let toml_string = fs::read_to_string("pod.toml").map_err(|err| format!("failed to open pod.toml ({})", err.to_string()))?;

    toml::from_str(&toml_string).map_err(|err| format!("failed to parse pod.toml ({})", err.to_string()))
}
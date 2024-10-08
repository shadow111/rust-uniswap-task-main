use crate::errors::UniswapError;

use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct Config {
	pub infura_endpoint_wss_url: String,
	pub contract_address: String,
}

impl Config {
	pub fn from_file(file_path: &str) -> Result<Self, UniswapError> {
		let config_string = fs::read_to_string(file_path)?;
		let config: Config = toml::from_str(&config_string)?;
		Ok(config)
	}
}

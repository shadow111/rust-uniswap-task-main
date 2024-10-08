mod client;
mod config;
mod errors;
mod event_subscriber;
mod io;
mod reorg_watcher;
mod swap_event;
mod swap_processor;

use crate::{
	client::Web3Client, config::Config, errors::UniswapError, event_subscriber::EventSubscriber,
	reorg_watcher::ReorgWatcher,
};
use log::warn;
use std::env;

#[tokio::main]
async fn main() -> Result<(), UniswapError> {
	let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| {
		warn!("CONFIG_PATH not set, using default 'config.toml'");
		"config.toml".to_string()
	});

	let config = Config::from_file(&config_path)?;
	let web3_client = Web3Client::new(&config.infura_endpoint_wss_url).await?;
	let mut event_subscriber = EventSubscriber::new(web3_client, &config).await?;
	let mut reorg_watcher = ReorgWatcher::new(6);

	event_subscriber.start(&mut reorg_watcher).await?;
	Ok(())
}

/*mod config;
mod errors;
mod eth_client;
mod io;
mod reorg_protection;
mod swap_event;

use crate::{config::Config, errors::UniswapError};
use anyhow::Result;
use eth_client::{EthereumClient, Web3Client};
use log::warn;
use reorg_protection::ReorgProtection;
use std::{env, sync::Arc};
use swap_event::SwapDetails;
use tokio::{sync::mpsc, task};
use web3::ethabi::Address;
use crate::io::load_contract_v1;

#[tokio::main]
async fn main() -> Result<(), UniswapError> {
	let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| {
		warn!("CONFIG_PATH not set, using default 'config.toml'");
		"config.toml".to_string()
	});
	let config = Config::from_file(&config_path)?;

	// Create Ethereum WebSocket client
	let client = Arc::new(Web3Client::new(&config.infura_endpoint_wss_url).await?);

	let contract_address: Address = config.contract_address.parse::<Address>().unwrap();

	let contract = load_contract_v1(Arc::clone(&client), contract_address)?;

	// Get the Swap event signature
	//let swap_event = contract.abi().event("Swap")?;
	//let swap_event_signature = swap_event.signature();

	// Create a channel for communication between event-fetcher and event-processor
	let (sender, mut receiver) = mpsc::channel(100); // Buffer size 100

	// Reorg protection buffer
	let mut reorg_protection = ReorgProtection::new(0);

	// Task 1: Event Fetching Task
	let client_clone = Arc::clone(&client);
	let sender_clone = sender.clone();

	task::spawn(async move {
		if let Err(err) = client_clone.subscribe_to_blocks(sender_clone).await {
			eprintln!("Error in WebSocket subscription: {:?}", err);
		}
	});

	// Task 2: Event Processing Task
	task::spawn(async move {
		while let Some(log) = receiver.recv().await {
			// Extract the block number from the log
			let block_number = log.block_number.unwrap_or_default();

			// Add the block number to reorg protection
			if let Err(err) = reorg_protection.add_block(block_number) {
				eprintln!("Reorg detected! {:?}", err);
				break; // Exit if deep reorg is detected
			}

			// For each log received, handle the swap event processing

			// If we have a confirmed block (N + 5), process its events
			if reorg_protection.is_confirmed() {
				if let Some(confirmed_block) = reorg_protection.confirmed_block() {
					println!("Processing confirmed block: {:?}", confirmed_block);

					// Process all logs for this confirmed block
					match SwapDetails::from_log(&log) {
						Ok(swap_details) => {
							// Handle the swap details
							println!("{:?}", swap_details);
						},
						Err(err) => {
							eprintln!("Failed to process log: {:?}", err);
						},
					}
				}
			}
		}
	});

	Ok(())

	// Sleep indefinitely to keep the tasks running
	/*loop {
		tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
	}*/
}*/

mod config;
mod errors;
mod io;
mod swap_event;

use crate::{
	config::Config,
	errors::UniswapError,
	io::{load_contract, Web3Socket},
	swap_event::SwapDetails,
};
use log::warn;
use std::{collections::VecDeque, env};
use web3::{
	futures::StreamExt,
	types::{Address, Block, BlockId, BlockNumber, FilterBuilder, H256},
};

#[tokio::main]
async fn main() -> Result<(), UniswapError> {
	let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| {
		warn!("CONFIG_PATH not set, using default 'config.toml'");
		"config.toml".to_string()
	});

	let config = Config::from_file(&config_path)?;

	let web3 = web3::Web3::new(
		Web3Socket::new(&config.infura_endpoint_wss_url)
			.await
			.map_err(|err| UniswapError::Web3Error(err.to_string()))?,
	);

	// Parse contract address
	let contract_address: Address = config.contract_address.parse::<Address>()
		.map_err(|e| UniswapError::ParseError(format!("Invalid contract address: {}", e)))?;


	let contract = load_contract(&web3, contract_address)?;

	// Get the Swap event signature
	let swap_event = contract.abi().event("Swap")?;
	let swap_event_signature = swap_event.signature();

	// Subscribe to new block headers
	let mut block_stream = web3
		.eth_subscribe()
		.subscribe_new_heads()
		.await
		.map_err(|err| UniswapError::Web3Error(err.to_string()))?;

	// Buffer to hold blocks for reorg protection
	let mut block_buffer: VecDeque<Block<H256>> = VecDeque::new();

	while let Some(Ok(block_header)) = block_stream.next().await {
		// Ensure block_header.hash and block_header.number are Some
		if block_header.hash.is_none() || block_header.number.is_none() {
			continue; // Skip this block
		}

		// Fetch the full block
		let block_number = block_header.number.unwrap();
		let block_id = BlockId::Number(BlockNumber::Number(block_number));
		let full_block = web3
			.eth()
			.block(block_id)
			.await
			.map_err(|err| UniswapError::Web3Error(err.to_string()))?;

		if let Some(full_block) = full_block {
			// Ensure full_block.hash is Some
			if full_block.hash.is_none() {
				continue; // Skip this block
			}

			block_buffer.push_back(full_block);

			// Ensure buffer length does not exceed 6 (N + 5 blocks)
			if block_buffer.len() > 6 {
				block_buffer.pop_front();
			}
			// Check for reorganizations
			if let Err(reorg_depth) = check_for_reorg(&block_buffer) {
				if reorg_depth > 5 {
					eprintln!("Reorganization with depth greater than 5 detected. Exiting.");
					std::process::exit(1);
				} else {
					eprintln!(
						"Reorganization detected with depth {}. Adjusting block buffer.",
						reorg_depth
					);
					// Remove invalid blocks from buffer
					for _ in 0..reorg_depth {
						block_buffer.pop_back();
					}
					continue;
				}
			}
			// Process the block at position 0 when we have N + 5 blocks
			if block_buffer.len() == 6 {
				let block_to_process = block_buffer[0].clone();

				// Fetch swap logs in the block
				if let Some(block_hash) = block_to_process.hash {
					let swap_logs_in_block = web3
						.eth()
						.logs(
							FilterBuilder::default()
								.block_hash(block_hash)
								.address(vec![contract_address])
								.topics(Some(vec![swap_event_signature]), None, None, None)
								.build(),
						)
						.await
						.map_err(|err| UniswapError::Web3Error(err.to_string()))?;

					for log in swap_logs_in_block {
						// Parse the log and extract swap details
						process_swap_log(log)?;
					}
				}
			}
		} else {
			continue; // Skip if full block is None
		}
	}

	Ok(())
}

// Function to check for reorganizations
fn check_for_reorg(block_buffer: &VecDeque<Block<H256>>) -> Result<(), usize> {
	for i in (1..block_buffer.len()).rev() {
		let current_block = &block_buffer[i];
		let prev_block = &block_buffer[i - 1];

		if let Some(prev_hash) = prev_block.hash {
			if current_block.parent_hash != prev_hash {
				let reorg_depth = block_buffer.len() - i;
				return Err(reorg_depth);
			}
		} else {
			// Cannot verify block linkage, treat as reorg
			let reorg_depth = block_buffer.len() - i;
			return Err(reorg_depth);
		}
	}
	Ok(())
}

// Function to process and print swap log details
fn process_swap_log(log: web3::types::Log) -> Result<(), UniswapError> {
	println!("-----------------------------");

	match SwapDetails::from_log(&log) {
		Ok(swap_details) => {
			// Handle the swap details
			println!("{:?}", swap_details);
		},
		Err(err) => {
			eprintln!("Failed to process log: {:?}", err);
		},
	}

	Ok(())
}

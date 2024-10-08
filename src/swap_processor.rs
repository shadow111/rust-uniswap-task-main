use crate::{client::Web3Client, errors::UniswapError, swap_event::SwapDetails};
use web3::types::{Address, Block, FilterBuilder, Log, H256};

pub struct SwapProcessor<'a> {
	client: &'a Web3Client,
	contract_address: Address,
	swap_event_signature: H256,
}

impl<'a> SwapProcessor<'a> {
	pub async fn new(
		client: &'a Web3Client,
		contract_address: Address,
		swap_event_signature: H256,
	) -> Result<Self, UniswapError> {
		Ok(Self { client, contract_address, swap_event_signature })
	}

	pub async fn process_block(&self, block: Block<H256>) -> Result<(), UniswapError> {
		if let Some(block_hash) = block.hash {
			let swap_logs = self.fetch_swap_logs(block_hash).await?;
			for log in swap_logs {
				self.process_swap_log(log)?;
			}
		}
		Ok(())
	}

	async fn fetch_swap_logs(&self, block_hash: H256) -> Result<Vec<Log>, UniswapError> {
		let filter = FilterBuilder::default()
			.block_hash(block_hash)
			.address(vec![self.contract_address])
			.topics(Some(vec![self.swap_event_signature]), None, None, None)
			.build();

		self.client
			.web3
			.eth()
			.logs(filter)
			.await
			.map_err(|e| UniswapError::Web3Error(e.to_string()))
	}

	pub fn process_swap_log(&self, log: Log) -> Result<(), UniswapError> {
		println!("-----------------------------");

		match SwapDetails::from_log(&log) {
			Ok(swap_details) => {
				println!("Swap Event Details:");
				println!("  Direction       : {:?}", swap_details.direction);
				println!("  DAI Amount      : {}", swap_details.dai_amount);
				println!("  USDC Amount     : {}", swap_details.usdc_amount);
				println!("  Sender          : {}", swap_details.sender);
				println!("  Recipient       : {}", swap_details.recipient);
				println!("-----------------------------");
			},
			Err(err) => {
				eprintln!("Failed to process log: {:?}", err);
			},
		}

		Ok(())
	}
}

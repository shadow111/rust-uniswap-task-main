use crate::{
	client::Web3Client, config::Config, errors::UniswapError, io::load_contract,
	reorg_watcher::ReorgWatcher, swap_processor::SwapProcessor,
};
use web3::{
	ethabi::Address,
	futures::StreamExt,
	types::{BlockId, BlockNumber, H256},
};

pub struct EventSubscriber {
	client: Web3Client,
	contract_address: Address,
	swap_event_signature: H256,
}

impl EventSubscriber {
	pub async fn new(client: Web3Client, config: &Config) -> Result<Self, UniswapError> {
		let contract_address = config
			.contract_address
			.parse::<Address>()
			.map_err(|e| UniswapError::ParseError(format!("Invalid contract address: {}", e)))?;

		let contract = load_contract(&client.web3, contract_address)?;
		let swap_event = contract.abi().event("Swap")?;
		let swap_event_signature = swap_event.signature();

		Ok(Self { client, contract_address, swap_event_signature })
	}

	pub async fn start(&mut self, reorg_watcher: &mut ReorgWatcher) -> Result<(), UniswapError> {
		let swap_processor =
			SwapProcessor::new(&self.client, self.contract_address, self.swap_event_signature)
				.await?;

		let mut block_stream = self
			.client
			.web3
			.eth_subscribe()
			.subscribe_new_heads()
			.await
			.map_err(|err| UniswapError::Web3Error(err.to_string()))?;

		while let Some(Ok(block_header)) = block_stream.next().await {
			if let Some(block_number) = block_header.number {
				let block_id = BlockId::Number(BlockNumber::Number(block_number));
				if let Some(full_block) = self.client.get_block(block_id).await? {
					reorg_watcher.add_block(full_block)?;

					if reorg_watcher.is_ready_to_process() {
						let block_to_process = reorg_watcher.get_block_to_process()?;
						swap_processor.process_block(block_to_process).await?;
					}
				}
			}
		}
		Ok(())
	}
}

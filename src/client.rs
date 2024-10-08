use crate::errors::UniswapError;
use web3::{
	transports::WebSocket,
	types::{Block, BlockId, H256},
	Web3,
};

pub struct Web3Client {
	pub web3: Web3<WebSocket>,
}

impl Web3Client {
	pub async fn new(wss_url: &str) -> Result<Self, UniswapError> {
		let transport = WebSocket::new(wss_url)
			.await
			.map_err(|e| UniswapError::Web3Error(e.to_string()))?;
		Ok(Self { web3: Web3::new(transport) })
	}

	pub async fn get_block(&self, block_id: BlockId) -> Result<Option<Block<H256>>, UniswapError> {
		self.web3
			.eth()
			.block(block_id)
			.await
			.map_err(|e| UniswapError::Web3Error(e.to_string()))
	}
}

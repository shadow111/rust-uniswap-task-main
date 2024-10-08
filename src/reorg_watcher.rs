use crate::errors::UniswapError;
use std::collections::VecDeque;
use web3::types::{Block, H256};

pub struct ReorgWatcher {
	block_buffer: VecDeque<Block<H256>>,
	buffer_size: usize,
}

impl ReorgWatcher {
	pub fn new(buffer_size: usize) -> Self {
		Self { block_buffer: VecDeque::with_capacity(buffer_size), buffer_size }
	}

	pub fn add_block(&mut self, block: Block<H256>) -> Result<(), UniswapError> {
		self.block_buffer.push_back(block);
		if self.block_buffer.len() > self.buffer_size {
			self.block_buffer.pop_front();
		}

		self.check_for_reorg()?;
		Ok(())
	}

	fn check_for_reorg(&self) -> Result<(), UniswapError> {
		for i in (1..self.block_buffer.len()).rev() {
			let current_block = &self.block_buffer[i];
			let prev_block = &self.block_buffer[i - 1];

			if let Some(prev_hash) = prev_block.hash {
				if current_block.parent_hash != prev_hash {
					let reorg_depth = self.block_buffer.len() - i;
					return Err(UniswapError::ReorgError(reorg_depth));
				}
			}
		}
		Ok(())
	}

	pub fn is_ready_to_process(&self) -> bool {
		self.block_buffer.len() == self.buffer_size
	}

	pub fn get_block_to_process(&self) -> Result<Block<H256>, UniswapError> {
		Ok(self
			.block_buffer
			.front()
			.cloned()
			.ok_or(UniswapError::BlockError("".to_string()))?)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use web3::types::{Block, H256};

	fn create_block(hash: u64, parent_hash: u64) -> Block<H256> {
		Block {
			hash: Some(H256::from_low_u64_be(hash)),
			parent_hash: H256::from_low_u64_be(parent_hash),
			..Default::default()
		}
	}

	#[test]
	fn test_reorg_no_reorg_detected() {
		let mut watcher = ReorgWatcher::new(6);

		// Create a chain of valid blocks
		let blocks = vec![
			create_block(1, 0),
			create_block(2, 1),
			create_block(3, 2),
			create_block(4, 3),
			create_block(5, 4),
			create_block(6, 5),
		];

		for block in blocks {
			let _ = watcher.add_block(block);
		}

		assert!(watcher.check_for_reorg().is_ok());
	}

	#[test]
	fn test_reorg_detected() {
		let mut watcher = ReorgWatcher::new(6);

		// Create a valid chain, then introduce a block with a wrong parent_hash (reorg)
		let blocks = vec![
			create_block(1, 0),
			create_block(2, 1),
			create_block(3, 2),
			create_block(4, 3),
			create_block(5, 4),
			create_block(6, 999), // Invalid parent_hash triggers reorg
		];

		for block in blocks {
			let _ = watcher.add_block(block);
		}

		let reorg_result = watcher.check_for_reorg();
		assert!(reorg_result.is_err());
		if let Err(UniswapError::ReorgError(reorg_depth)) = reorg_result {
			assert_eq!(reorg_depth, 1); // Reorg at depth 1 (invalid last block)
		}
	}

	#[test]
	fn test_reorg_large_depth_detected() {
		let mut watcher = ReorgWatcher::new(6);

		// Introduce a reorg at a larger depth (middle of the chain)
		let blocks = vec![
			create_block(1, 0),
			create_block(2, 1),
			create_block(3, 2), // Invalid parent_hash triggers reorg at depth 3
			create_block(4, 999),
			create_block(5, 4),
			create_block(6, 5),
		];

		for block in blocks {
			let _ = watcher.add_block(block);
		}

		let reorg_result = watcher.check_for_reorg();
		assert!(reorg_result.is_err());
		if let Err(UniswapError::ReorgError(reorg_depth)) = reorg_result {
			assert_eq!(reorg_depth, 3); // Reorg at depth 3 (invalid block 3)
		}
	}
}

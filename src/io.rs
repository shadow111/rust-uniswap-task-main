use crate::errors::UniswapError;
use web3::{contract::Contract, types::Address, Web3};

pub type Web3Socket = web3::transports::WebSocket;
pub fn load_contract(
	web3: &Web3<Web3Socket>,
	contract_address: Address,
) -> Result<Contract<Web3Socket>, UniswapError> {
	let abi_bytes = include_bytes!("contracts/uniswap_pool_abi.json");
	let contract = Contract::from_json(web3.eth(), contract_address, &abi_bytes[..])?;

	Ok(contract)
}

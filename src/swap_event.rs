use bigdecimal::{num_bigint::BigInt, BigDecimal, Zero};
//use num_bigint::BigInt;
//use num_traits::Zero;

use crate::errors::UniswapError;
use serde::{Deserialize, Serialize};
use web3::ethabi::Address;

#[derive(Debug)]
pub struct SwapDetails {
	pub dai_amount: String,
	pub usdc_amount: String,
	pub direction: SwapDirection,
	pub sender: String,
	pub recipient: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SwapDirection {
	DaiToUsdc,
	UsdcToDai,
}

impl SwapDetails {
	pub fn from_log(log: &web3::types::Log) -> Result<Self, UniswapError> {
		let data = &log.data.0;

		let amount0_in = BigInt::from_signed_bytes_be(&data[0..32]);
		let amount1_out = BigInt::from_signed_bytes_be(&data[32..64]);

		let sender = format!("{:?}", Address::from(log.topics[1]));
		let recipient = format!("{:?}", Address::from(log.topics[2]));

		let direction = if amount0_in > BigInt::zero() {
			SwapDirection::DaiToUsdc
		} else {
			SwapDirection::UsdcToDai
		};

		let dai_amount = convert_dai(amount0_in).to_string();
		let usdc_amount = convert_usdc(amount1_out).to_string();

		Ok(SwapDetails { dai_amount, usdc_amount, direction, sender, recipient })
	}
}

fn convert_dai(amount: BigInt) -> BigDecimal {
	BigDecimal::new(amount, 18)
}

fn convert_usdc(amount: BigInt) -> BigDecimal {
	BigDecimal::new(amount, 6)
}

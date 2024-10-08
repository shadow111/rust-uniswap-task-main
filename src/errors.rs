use std::{fmt, io};

#[derive(Debug)]
pub enum UniswapError {
	IoError(io::Error),
	InvalidAbi(web3::ethabi::Error),
	Web3Error(String),
	ConfError(toml::de::Error),
	ParseError(String),
	ReorgError(usize),
	BlockError(String),
}

impl fmt::Display for UniswapError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			UniswapError::IoError(err) => write!(f, "IoError: {}", err),
			UniswapError::InvalidAbi(err) => write!(f, "InvalidAbi: {}", err),
			UniswapError::Web3Error(err) => write!(f, "Web3Error: {}", err),
			UniswapError::ConfError(err) => write!(f, "ConfError: {}", err),
			UniswapError::ParseError(err) => write!(f, "ParseError: {}", err),
			UniswapError::ReorgError(err) => write!(f, "ReorgError: {}", err),
			UniswapError::BlockError(err) => write!(f, "BlockError: {}", err),
		}
	}
}

impl From<io::Error> for UniswapError {
	fn from(err: io::Error) -> UniswapError {
		UniswapError::IoError(err)
	}
}

impl From<web3::ethabi::Error> for UniswapError {
	fn from(err: web3::ethabi::Error) -> UniswapError {
		UniswapError::InvalidAbi(err)
	}
}

impl From<toml::de::Error> for UniswapError {
	fn from(err: toml::de::Error) -> UniswapError {
		UniswapError::ConfError(err)
	}
}

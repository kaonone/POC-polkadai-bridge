use primitives::{crypto::Pair, sr25519};
use rustc_hex::FromHex;
use web3::types::Address;

use raw_transaction_builder::Bip32ECKeyPair;

use std::env;

const DEFAULT_GAS_PRICE: u64 = 24_000_000_000;
const DEFAULT_GAS: u64 = 5_000_000;

#[derive(Clone, Debug)]
pub struct Config {
    pub graph_node_api_url: String,
    pub eth_api_url: String,
    pub eth_validator_address: Address,
    pub eth_validator_private_key: String,
    pub eth_contract_address: Address,
    pub eth_gas_price: u64,
    pub eth_gas: u64,
    pub sub_api_url: String,
    pub sub_validator_mnemonic_phrase: String,
}

impl Config {
    pub fn load() -> Result<Self, &'static str> {
        Ok(Config {
            graph_node_api_url: parse_graph_node_api_url()?,
            eth_api_url: parse_eth_api_url()?,
            eth_validator_address: parse_eth_validator_address()?,
            eth_validator_private_key: parse_eth_validator_private_key()?,
            eth_contract_address: parse_eth_contract_address()?,
            eth_gas_price: parse_eth_gas_price()?,
            eth_gas: parse_eth_gas()?,
            sub_api_url: parse_sub_api_url()?,
            sub_validator_mnemonic_phrase: parse_sub_validator_mnemonic_phrase()?,
        })
    }
}

fn parse_graph_node_api_url() -> Result<String, &'static str> {
    env::var("GRAPH_NODE_API_URL").map_err(|_| "can not read GRAPH_NODE_API_URL")
}

fn parse_eth_api_url() -> Result<String, &'static str> {
    env::var("ETH_API_URL").map_err(|_| "can not read ETH_API_URL")
}

fn parse_eth_validator_address() -> Result<Address, &'static str> {
    let address =
        env::var("ETH_VALIDATOR_ADDRESS").map_err(|_| "can not read ETH_VALIDATOR_ADDRESS")?;
    address[2..]
        .parse()
        .map_err(|_| "can not parse validator address")
}

fn parse_eth_validator_private_key() -> Result<String, &'static str> {
    let private_key = env::var("ETH_VALIDATOR_PRIVATE_KEY")
        .map_err(|_| "can not read ETH_VALIDATOR_PRIVATE_KEY")?;
    let private_key = private_key[2..].to_string();
    try_convert_to_bip32_key_pair(&private_key)?;

    Ok(private_key)
}

fn parse_eth_contract_address() -> Result<Address, &'static str> {
    let address =
        env::var("ETH_CONTRACT_ADDRESS").map_err(|_| "can not read ETH_CONTRACT_ADDRESS")?;
    address[2..]
        .parse()
        .map_err(|_| "can not parse contract address")
}

fn parse_eth_gas_price() -> Result<u64, &'static str> {
    env::var("ETH_GAS_PRICE")
        .or_else(|_| Ok(DEFAULT_GAS_PRICE.to_string()))
        .map(|x| x.parse().expect("can not parse ETH_GAS_PRICE"))
}

fn parse_eth_gas() -> Result<u64, &'static str> {
    env::var("ETH_GAS")
        .or_else(|_| Ok(DEFAULT_GAS.to_string()))
        .map(|x| x.parse().expect("can not parse ETH_GAS"))
}

fn parse_sub_api_url() -> Result<String, &'static str> {
    env::var("SUB_API_URL").map_err(|_| "can not read SUB_API_URL")
}

fn parse_sub_validator_mnemonic_phrase() -> Result<String, &'static str> {
    let mnemonic_phrase = env::var("SUB_VALIDATOR_MNEMONIC_PHRASE")
        .map_err(|_| "can not read SUB_VALIDATOR_MNEMONIC_PHRASE")?;
    try_convert_to_sr25519_key_pair(&mnemonic_phrase)?;

    Ok(mnemonic_phrase)
}

fn try_convert_to_sr25519_key_pair(mnemonic_phrase: &str) -> Result<(), &'static str> {
    sr25519::Pair::from_phrase(&mnemonic_phrase, None)
        .map_err(|_| "invalid SUB_VALIDATOR_MNEMONIC_PHRASE")?;
    Ok(())
}

fn try_convert_to_bip32_key_pair(private_key: &str) -> Result<(), &'static str> {
    let private_key = private_key
        .from_hex::<Vec<_>>()
        .map_err(|_| "can not parse validator private key")?;
    Bip32ECKeyPair::from_raw_secret(&private_key).map_err(|_| "invalid validator private key")?;
    Ok(())
}

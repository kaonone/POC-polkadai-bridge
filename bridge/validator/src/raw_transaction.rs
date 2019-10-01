use raw_transaction_builder::{Bip32ECKeyPair, RawTransaction};
use rustc_hex::FromHex;
use web3::{self, types::U256};

const CHAIN_ID: u8 = 42;

pub fn build(
    private_key: String,
    to: web3::types::H160,
    nonce: web3::types::U256,
    value: u64,
    gas_price: u64,
    gas: u64,
    data: Vec<u8>,
) -> Vec<u8> {
    let tx = RawTransaction {
        nonce,
        to: Some(to),
        value: U256::from(value),
        gas_price: U256::from(gas_price),
        gas_limit: U256::from(gas),
        data,
    };

    let bip32ec_keypair = Bip32ECKeyPair::from_raw_secret(
        &private_key
            .from_hex::<Vec<_>>()
            .expect("can not parse private key"),
    )
    .expect("invalid private key");
    tx.sign(&bip32ec_keypair, CHAIN_ID)
}

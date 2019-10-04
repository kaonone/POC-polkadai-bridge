use ethabi::{self, ParamType};
use futures::{
    future::{lazy, poll_fn},
    stream::Stream,
};
use log;
use tokio_threadpool::blocking;
use web3::{
    futures::Future,
    types::{Bytes, Filter, FilterBuilder, Log},
};

use primitives::sr25519;
use substrate_api_client::Api;

use std::sync::Arc;

use crate::config;
use crate::ethereum_transactions;
use crate::substrate_transactions;

const AMOUNT: u64 = 0;

pub fn start(config: config::Config) {
    let mut sub_api = Api::new(config.sub_api_url.clone());
    sub_api.init();
    let sub_api = Arc::new(sub_api);

    let (_eloop, transport) = web3::transports::WebSocket::new(&config.eth_api_url).unwrap();
    let web3 = web3::Web3::new(transport);

    let contact_abi = include_bytes!("../res/EthContract.abi");
    let abi = ethabi::Contract::load(contact_abi.to_vec().as_slice()).expect("can read ABI");

    let abi = Arc::new(abi);
    let web3 = Arc::new(web3);

    let fut = web3
        .eth_subscribe()
        .subscribe_logs(build_filter(&config))
        .then(move |sub| {
            sub.unwrap().for_each(move |log| {
                log::info!("[ethereum] got log: {:?}", log);
                let received_relay_message = log
                    .topics
                    .iter()
                    .any(|addr| addr == &config.eth_relay_message_hash);
                let received_approved_relay_message = log
                    .topics
                    .iter()
                    .any(|addr| addr == &config.eth_approved_relay_message_hash);
                let withdraw_message = log
                    .topics
                    .iter()
                    .any(|addr| addr == &config.eth_withdraw_message_hash);

                match (
                    received_relay_message,
                    received_approved_relay_message,
                    withdraw_message,
                ) {
                    (true, _, _) => handle_relay_message(log, web3.clone(), abi.clone(), &config),
                    (_, true, _) => handle_approved_relay_message(log, sub_api.clone(), &config),
                    (_, _, true) => handle_withdraw_message(log, sub_api.clone(), &config),
                    (_, _, _) => {
                        log::warn!("received unknown log: {:?}", log);
                        Ok(())
                    }
                }
            })
        })
        .map_err(|_| ());

    tokio::run(fut);
}

fn build_filter(config: &config::Config) -> Filter {
    FilterBuilder::default()
        .address(vec![config.eth_contract_address])
        .topics(
            Some(vec![
                config.eth_relay_message_hash,
                config.eth_approved_relay_message_hash,
                config.eth_withdraw_message_hash,
            ]),
            None,
            None,
            None,
        )
        .build()
}

fn handle_relay_message<T>(
    log: Log,
    web3: Arc<web3::Web3<T>>,
    abi: Arc<ethabi::Contract>,
    config: &config::Config,
) -> Result<(), web3::error::Error>
where
    T: web3::Transport + Send + Sync + 'static,
    T::Out: Send,
{
    let result = ethabi::decode(
        &[
            ParamType::FixedBytes(32),
            ParamType::Address,
            ParamType::FixedBytes(32),
            ParamType::Uint(256),
        ],
        &log.data.0,
    );
    if let Ok(params) = result {
        log::info!("[ethereum] got decoded log.data: {:?}", params);
        if params.len() >= 4 {
            let args = (
                params[0].clone(),
                params[1].clone(),
                params[2].clone(),
                params[3].clone(),
            );

            let eth_validator_private_key = config.eth_validator_private_key.clone();
            let eth_contract_address = config.eth_contract_address;
            let eth_gas_price = config.eth_gas_price;
            let eth_gas = config.eth_gas;
            let data = ethereum_transactions::build_transaction_data(&abi, "approveTransfer", args);
            let fut = web3.eth().transaction_count(config.eth_validator_address, None)
                .and_then(move |nonce| {
                    let tx = ethereum_transactions::build(eth_validator_private_key, eth_contract_address, nonce, AMOUNT, eth_gas_price, eth_gas, data);
                    log::debug!("raw approveTransfer: {:?}", tx);
                    web3.eth().send_raw_transaction(Bytes::from(tx))
                        .then(move |res| {
                            match res {
                                Ok(tx_res) => {
                                    log::info!("[ethereum] called approveTransfer({:?}, {:?}, {:?}, {:?}), nonce: {:?}, result: {:?}",
                                                params[0], params[1], params[2], params[3], nonce, tx_res);
                                },
                                Err(err) => {
                                    log::warn!("[ethereum] can not send approveTransfer({:?}, {:?}, {:?}, {:?}), nonce: {:?}, reason: {:?}",
                                                params[0], params[1], params[2], params[3], nonce, err);
                                }
                            }
                            Ok(())
                        })

                })
                .map_err(|e| log::warn!("can not get nonce: {:?}", e));
            tokio::spawn(fut);
        }
    }
    Ok(())
}

fn handle_approved_relay_message(
    log: Log,
    sub_api: Arc<Api>,
    config: &config::Config,
) -> Result<(), web3::error::Error> {
    let result = ethabi::decode(
        &[
            ParamType::FixedBytes(32),
            ParamType::Address,
            ParamType::FixedBytes(32),
            ParamType::Uint(256),
        ],
        &log.data.0,
    );
    if let Ok(params) = result {
        log::info!("[ethereum] got decoded log.data: {:?}", params);
        if params.len() >= 4 {
            let message_id = params[0]
                .clone()
                .to_fixed_bytes()
                .map(|x| primitives::H256::from_slice(&x))
                .expect("can not parse message_id");
            let from = params[1]
                .clone()
                .to_address()
                .map(|x| primitives::H160::from(x.as_fixed_bytes()))
                .expect("can not parse 'from' address");
            let to = params[2]
                .clone()
                .to_fixed_bytes()
                .map(|x| sr25519::Public::from_slice(&x))
                .expect("can not parse 'to' address");
            let amount = params[3]
                .clone()
                .to_uint()
                .map(|x| x.low_u64())
                .expect("can not parse amount");

            let sub_validator_mnemonic_phrase = config.sub_validator_mnemonic_phrase.clone();
            let sub_api = sub_api.clone();
            tokio::spawn(lazy(move || {
                poll_fn(move || {
                    blocking(|| {
                        substrate_transactions::mint(
                            sub_api.clone(),
                            sub_validator_mnemonic_phrase.clone(),
                            message_id,
                            from,
                            to.clone(),
                            amount,
                        );
                        log::info!(
                            "[substrate] called multi_signed_mint({:?}, {:?}, {:?}, {:?})",
                            message_id,
                            from,
                            to,
                            amount
                        );
                    })
                    .map_err(|_| panic!("the threadpool shut down"))
                })
            }));
        }
    }
    Ok(())
}

fn handle_withdraw_message(
    log: Log,
    sub_api: Arc<Api>,
    config: &config::Config,
) -> Result<(), web3::error::Error> {
    let result = ethabi::decode(&[ParamType::FixedBytes(32)], &log.data.0);
    if let Ok(params) = result {
        log::info!("[ethereum] got decoded log.data: {:?}", params);
        if params.len() >= 4 {
            let message_id = params[0]
                .clone()
                .to_fixed_bytes()
                .map(|x| primitives::H256::from_slice(&x))
                .expect("can not parse message_id");

            let sub_validator_mnemonic_phrase = config.sub_validator_mnemonic_phrase.clone();
            let sub_api = sub_api.clone();
            tokio::spawn(lazy(move || {
                poll_fn(move || {
                    blocking(|| {
                        substrate_transactions::confirm_transfer(
                            &sub_api,
                            sub_validator_mnemonic_phrase.clone(),
                            message_id,
                        );
                        log::info!("[substrate] called confirm_transfer({:?})", message_id);
                    })
                    .map_err(|_| panic!("the threadpool shut down"))
                })
            }));
        }
    }
    Ok(())
}

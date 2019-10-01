use dotenv::dotenv;
use env_logger;
use ethabi::{self, ParamType};
use futures::{
    future::{lazy, poll_fn},
    stream::Stream,
};
use log;
use rustc_hex::FromHex;
use tokio_threadpool::blocking;
use web3::{
    contract::{
        tokens::{Tokenizable, Tokenize},
        Contract,
    },
    futures::Future,
    types::FilterBuilder,
    types::{Address, Bytes, H160, H256, U256},
};

use node_runtime::{bridge, AccountId, Event};
use parity_codec::Decode;
use primitives::{crypto::Pair, sr25519};
use substrate_api_client::{hexstr_to_vec, Api};
use system;

use std::env;
use std::sync::{mpsc, Arc};
use std::thread;

use raw_transaction_builder::Bip32ECKeyPair;

mod extrinsics;
mod raw_transaction;

const AMOUNT: u64 = 0;
const GAS_PRICE: u64 = 24_000_000_000;
const GAS: u64 = 5_000_000;

fn main() {
    env_logger::init();
    dotenv().ok();

    let (
        eth_api_url,
        eth_validator_address,
        eth_validator_private_key,
        eth_contract_address,
        eth_relay_message_hash,
        eth_approved_relay_message_hash,
        eth_withdraw_message_hash,
        sub_api_url,
        sub_validator_mnemonic_phrase,
    ) = read_env();

    log::info!("[ethereum] api url: {:?}", eth_api_url);
    log::info!("[ethereum] validator address: {:?}", eth_validator_address);
    log::info!("[ethereum] contract address: {:?}", eth_contract_address);
    log::info!(
        "[ethereum] hash of RelayMessage: {:?}",
        eth_relay_message_hash
    );
    log::info!(
        "[ethereum] hash of ApprovedRelayMessage: {:?}",
        eth_approved_relay_message_hash
    );
    log::info!("[substrate] api url: {:?}", sub_api_url);

    let (_event_subscriber, _event_handler) = start_substrate_event_handler(
        sub_api_url.clone(),
        sub_validator_mnemonic_phrase.clone(),
        eth_api_url.clone(),
        eth_validator_address,
        eth_validator_private_key.clone(),
        eth_contract_address,
    );

    let mut sub_api = Api::new(sub_api_url);
    sub_api.init();
    let sub_api = Arc::new(sub_api);

    let (_eloop, transport) = web3::transports::WebSocket::new(&eth_api_url).unwrap();
    let web3 = web3::Web3::new(transport);

    let contact_abi = include_bytes!("../res/EthContract.abi");
    let contract = Contract::from_json(web3.eth(), eth_contract_address, contact_abi)
        .expect("can not create contract");
    let abi = ethabi::Contract::load(contact_abi.to_vec().as_slice()).expect("can read ABI");
    let web3 = Arc::new(web3);

    let filter = FilterBuilder::default()
        .address(vec![contract.address()])
        .topics(
            Some(vec![
                eth_relay_message_hash,
                eth_approved_relay_message_hash,
                eth_withdraw_message_hash,
            ]),
            None,
            None,
            None,
        )
        .build();

    let fut = web3
        .eth_subscribe()
        .subscribe_logs(filter)
        .then(move |sub| {
            sub.unwrap().for_each(move |log| {
                log::info!("[ethereum] got log: {:?}", log);
                let received_relay_message = log.topics.iter().any(|addr| addr == &eth_relay_message_hash);
                let received_approved_relay_message = log.topics.iter().any(|addr| addr == &eth_approved_relay_message_hash);
                let withdraw_message = log.topics.iter().any(|addr| addr == &eth_withdraw_message_hash);

                match (received_relay_message, received_approved_relay_message, withdraw_message) {
                    (true, _, _) => {
                        let result = ethabi::decode(&[ParamType::FixedBytes(32), ParamType::Address, ParamType::FixedBytes(32), ParamType::Uint(256)], &log.data.0);
                        if let Ok(params) = result {
                            log::info!("[ethereum] got decoded log.data: {:?}", params);
                            if params.len() >= 4 {
                                let args = (params[0].clone(), params[1].clone(), params[2].clone(), params[3].clone());

                                let web3 = web3.clone();
                                let eth_validator_private_key =  eth_validator_private_key.clone();
                                let data = build_transaction_data(&abi, "approveTransfer", args);
                                let fut = web3.eth().transaction_count(eth_validator_address, None)
                                    .and_then(move |nonce| {
                                        let tx = raw_transaction::build(eth_validator_private_key, eth_contract_address, nonce, AMOUNT, GAS_PRICE, GAS, data);
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
                    },
                    (_, true, _) => {
                        let result = ethabi::decode(&[ParamType::FixedBytes(32), ParamType::Address, ParamType::FixedBytes(32), ParamType::Uint(256)], &log.data.0);
                        if let Ok(params) = result {
                            log::info!("[ethereum] got decoded log.data: {:?}", params);
                            if params.len() >= 4 {
                                let message_id = params[0].clone().to_fixed_bytes().map(|x| primitives::H256::from_slice(&x)).expect("can not parse message_id");
                                let from = params[1].clone().to_address().map(|x| primitives::H160::from(x.as_fixed_bytes())).expect("can not parse 'from' address");
                                let to = params[2].clone().to_fixed_bytes().map(|x| sr25519::Public::from_slice(&x)).expect("can not parse 'to' address");
                                let amount = params[3].clone().to_uint().map(|x| x.low_u64()).expect("can not parse amount");

                                let sub_validator_mnemonic_phrase = sub_validator_mnemonic_phrase.clone();
                                let sub_api = sub_api.clone();
                                tokio::spawn(lazy(move || {
                                    poll_fn(move || {
                                        blocking(|| {
                                            mint(sub_api.clone(), sub_validator_mnemonic_phrase.clone(), message_id, from, to.clone(), amount);
                                            log::info!("[substrate] called multi_signed_mint({:?}, {:?}, {:?}, {:?})", message_id, from, to, amount);
                                        }).map_err(|_| panic!("the threadpool shut down"))
                                    })
                                }));
                            }
                        }
                        Ok(())
                    }
                    (_, _, true) => {
                        let result = ethabi::decode(&[ParamType::FixedBytes(32), ParamType::FixedBytes(32), ParamType::Address, ParamType::Uint(256)], &log.data.0);
                        if let Ok(params) = result {
                            log::info!("[ethereum] got decoded log.data: {:?}", params);
                            if params.len() >= 4 {
                                let message_id = params[0].clone().to_fixed_bytes().map(|x| primitives::H256::from_slice(&x)).expect("can not parse message_id");

                                let sub_validator_mnemonic_phrase = sub_validator_mnemonic_phrase.clone();
                                let sub_api = sub_api.clone();
                                tokio::spawn(lazy(move || {
                                    poll_fn(move || {
                                        blocking(|| {
                                            confirm_transfer(&sub_api, sub_validator_mnemonic_phrase.clone(), message_id);
                                            log::info!("[substrate] called confirm_transfer({:?})", message_id);
                                        }).map_err(|_| panic!("the threadpool shut down"))
                                    })
                                }));
                            }
                        }
                        Ok(())
                    }
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

fn build_transaction_data<P>(abi: &ethabi::Contract, function_name: &str, params: P) -> Vec<u8>
where
    P: Tokenize,
{
    abi.function(function_name)
        .and_then(|function| function.encode_input(&params.into_tokens()))
        .unwrap_or_else(|error| {
            log::warn!(
                "can not build transaction data for {:?}: {:?}",
                function_name,
                error
            );
            vec![]
        })
}

fn read_env() -> (
    String,
    Address,
    String,
    Address,
    H256,
    H256,
    H256,
    String,
    String,
) {
    let eth_api_url = env::var("ETH_API_URL").expect("can not read ETH_API_URL");
    let eth_validator_address =
        env::var("ETH_VALIDATOR_ADDRESS").expect("can not read ETH_VALIDATOR_ADDRESS");
    let eth_validator_private_key =
        env::var("ETH_VALIDATOR_PRIVATE_KEY").expect("can not read ETH_VALIDATOR_PRIVATE_KEY");
    let eth_contract_address =
        env::var("ETH_CONTRACT_ADDRESS").expect("can not read ETH_CONTRACT_ADDRESS");
    let eth_relay_message_hash =
        env::var("ETH_RELAY_MESSAGE_HASH").expect("can not read ETH_RELAY_MESSAGE_HASH");
    let eth_approved_relay_message_hash = env::var("ETH_APPROVED_RELAY_MESSAGE_HASH")
        .expect("can not read ETH_APPROVED_RELAY_MESSAGE_HASH");
    let eth_withdraw_message_hash =
        env::var("ETH_WITHDRAW_MESSAGE_HASH").expect("can not read ETH_WITHDRAW_MESSAGE_HASH");

    let sub_api_url = env::var("SUB_API_URL").expect("can not read SUB_API_URL");
    let sub_validator_mnemonic_phrase = env::var("SUB_VALIDATOR_MNEMONIC_PHRASE")
        .expect("can not read SUB_VALIDATOR_MNEMONIC_PHRASE");
    let _ = sr25519::Pair::from_phrase(&sub_validator_mnemonic_phrase, None)
        .expect("invalid SUB_VALIDATOR_MNEMONIC_PHRASE");
    let _ = Bip32ECKeyPair::from_raw_secret(
        &eth_validator_private_key[2..]
            .from_hex::<Vec<_>>()
            .expect("can not parse validator private key"),
    )
    .expect("invalid validator private key");

    (
        eth_api_url.to_string(),
        eth_validator_address[2..]
            .parse()
            .expect("can not parse validator address"),
        eth_validator_private_key[2..].to_string(),
        eth_contract_address[2..]
            .parse()
            .expect("can not parse contract address"),
        eth_relay_message_hash[2..]
            .parse()
            .expect("can not parse event hash"),
        eth_approved_relay_message_hash[2..]
            .parse()
            .expect("can not parse event hash"),
        eth_withdraw_message_hash[2..]
            .parse()
            .expect("can not parse event hash"),
        sub_api_url.to_string(),
        sub_validator_mnemonic_phrase,
    )
}

fn start_substrate_event_handler(
    api_url: String,
    signer_mnemonic_phrase: String,
    eth_api_url: String,
    eth_validator_address: H160,
    eth_validator_private_key: String,
    eth_contract_address: Address,
) -> (thread::JoinHandle<()>, thread::JoinHandle<()>) {
    let (events_in, events_out) = mpsc::channel();

    let event_subscriber = start_event_subscriber(api_url.clone(), events_in);
    let event_handler = start_event_handler(
        api_url,
        signer_mnemonic_phrase,
        eth_api_url,
        eth_validator_address,
        eth_validator_private_key,
        eth_contract_address,
        events_out,
    );

    (event_subscriber, event_handler)
}

fn start_event_subscriber(
    api_url: String,
    events_in: mpsc::Sender<String>,
) -> thread::JoinHandle<()> {
    let mut sub_api = Api::new(api_url);
    sub_api.init();

    thread::Builder::new()
        .name("event_subscriber".to_string())
        .spawn(move || {
            sub_api.subscribe_events(events_in.clone());
        })
        .expect("can not start event_subscriber")
}

fn start_event_handler(
    api_url: String,
    signer_mnemonic_phrase: String,
    eth_api_url: String,
    eth_validator_address: H160,
    eth_validator_private_key: String,
    eth_contract_address: Address,
    events_out: mpsc::Receiver<String>,
) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("event_handler".to_string())
        .spawn(move || {
            let mut sub_api = Api::new(api_url);
            sub_api.init();

            let (_eloop, transport) = web3::transports::WebSocket::new(&eth_api_url).unwrap();
            let web3 = web3::Web3::new(transport);

            let abi = ethabi::Contract::load(include_bytes!("../res/EthContract.abi").to_vec().as_slice()).expect("can read ABI");

            for event in events_out {
                log::debug!("[substrate] got event: {:?}", event);

                let unhex = hexstr_to_vec(event);
                let mut er_enc = unhex.as_slice();
                let events = Vec::<system::EventRecord::<Event>>::decode(&mut er_enc);

                match events {
                    Some(evts) => {
                        for evr in &evts {
                            log::debug!("[substrate] decoded: phase {:?} event {:?}", evr.phase, evr.event);
                            match &evr.event {
                                Event::bridge(br) => {
                                    log::info!("[substrate] bridge event: {:?}", br);
                                    match &br {
                                        bridge::RawEvent::RelayMessage(message_id) => {
                                            approve_transfer(&sub_api, signer_mnemonic_phrase.clone(), *message_id);
                                            log::info!("[substrate] called approve_transfer({:?})", message_id);
                                        },
                                        bridge::RawEvent::ApprovedRelayMessage(message_id, from, to, amount) => {
                                            let args = (
                                                message_id.as_fixed_bytes().into_token(),
                                                H256::from_slice(from.as_slice()).as_fixed_bytes().into_token(),
                                                Address::from(to.as_fixed_bytes()).into_token(),
                                                U256::from(*amount).into_token()
                                            );
                                            let web3 = web3.clone();
                                            let eth_validator_private_key =  eth_validator_private_key.clone();
                                            let data = build_transaction_data(&abi, "withdrawTransfer", args.clone());
                                            let fut = web3.eth().transaction_count(eth_validator_address, None)
                                                .and_then(move |nonce| {
                                                    let tx = raw_transaction::build(eth_validator_private_key, eth_contract_address, nonce, AMOUNT, GAS_PRICE, GAS, data);
                                                    log::debug!("raw withdrawTransfer: {:?}", tx);
                                                    web3.eth().send_raw_transaction(Bytes::from(tx))
                                                        .then(move |res| {
                                                            match res {
                                                                Ok(tx_res) => {
                                                                    log::info!("[ethereum] called withdrawTransfer({:?}, {:?}, {:?}, {:?}), nonce: {:?}, result: {:?}",
                                                                               args.0, args.1, args.2, args.3, nonce, tx_res)
                                                                },
                                                                Err(err) => {
                                                                    log::warn!("can not send withdrawTransfer({:?}, {:?}, {:?}, {:?}), nonce: {:?}, reason: {:?}",
                                                                               args.0, args.1, args.2, args.3, nonce, err);

                                                                }
                                                            }

                                                            Ok(())
                                                        })
                                                })
                                                .or_else(|e| {
                                                    log::warn!("can not get nonce: {:?}", e);
                                                    Ok(())
                                                });
                                            tokio::run(fut);
                                        },
                                        bridge::RawEvent::Burned(message_id, from, to, amount) => {
                                            log::info!("[substrate] received Burned({:?}, {:?}, {:?}, {:?})", message_id, from, to, amount);
                                        },
                                        bridge::RawEvent::Minted(message_id) => {
                                            let args = (
                                                H256::from(message_id.as_fixed_bytes()).into_token(),
                                            );

                                            let web3 = web3.clone();
                                            let eth_validator_private_key =  eth_validator_private_key.clone();
                                            let data = build_transaction_data(&abi, "confirmTransfer", args.clone());
                                            let fut = web3.eth().transaction_count(eth_validator_address, None)
                                                .and_then(move |nonce| {
                                                    let tx = raw_transaction::build(eth_validator_private_key, eth_contract_address, nonce, AMOUNT, GAS_PRICE, GAS, data);
                                                    log::debug!("raw confirmTransfer: {:?}", tx);
                                                    web3.eth().send_raw_transaction(Bytes::from(tx))
                                                        .then(move |res| {
                                                            match res {
                                                                Ok(tx_res) => {
                                                                    log::info!("[ethereum] called confirmTransfer({:?}), nonce: {:?}, result: {:?}",
                                                                               args.0, nonce, tx_res)
                                                                },
                                                                Err(err) => {
                                                                    log::info!("[ethereum] can not send confirmTransfer({:?}), nonce: {:?}, reason: {:?}",
                                                                               args.0, nonce, err)
                                                                }
                                                            }

                                                            Ok(())
                                                        })
                                                })
                                                .or_else(|e| {
                                                    log::warn!("can not get nonce: {:?}", e);
                                                    Ok(())
                                                });
                                            tokio::run(fut);
                                        },
                                        _ => {
                                            log::debug!("[substrate] ignoring unsupported balances event");
                                        },
                                    }
                                },
                                _ => {
                                    log::debug!("[substrate] ignoring unsupported module event: {:?}", evr.event)
                                    },
                            }
                        }
                    }
                    None => log::error!("[substrate] could not decode event record list")
                }
            }
        }).expect("can not start event_handler")
}

fn mint(
    sub_api: Arc<Api>,
    signer_mnemonic_phrase: String,
    message_id: primitives::H256,
    from: primitives::H160,
    to: AccountId,
    amount: u64,
) {
    let signer = sr25519::Pair::from_phrase(&signer_mnemonic_phrase, None)
        .expect("invalid menemonic phrase");
    let xthex = extrinsics::build_mint(&sub_api, signer, message_id, from, to, amount);

    //send and watch extrinsic until finalized
    let _tx_hash = sub_api.send_extrinsic(xthex);
}

fn approve_transfer(sub_api: &Api, signer_mnemonic_phrase: String, message_id: primitives::H256) {
    let signer = sr25519::Pair::from_phrase(&signer_mnemonic_phrase, None)
        .expect("invalid menemonic phrase");
    let xthex = extrinsics::build_approve_transfer(&sub_api, signer, message_id);

    //send and watch extrinsic until finalized
    let _tx_hash = sub_api.send_extrinsic(xthex);
}

fn confirm_transfer(sub_api: &Api, signer_mnemonic_phrase: String, message_id: primitives::H256) {
    let signer = sr25519::Pair::from_phrase(&signer_mnemonic_phrase, None)
        .expect("invalid menemonic phrase");
    let xthex = extrinsics::build_confirm_transfer(&sub_api, signer, message_id);

    //send and watch extrinsic until finalized
    let _tx_hash = sub_api.send_extrinsic(xthex);
}

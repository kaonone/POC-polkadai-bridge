use ethabi;
use log;
use web3::{
    contract::tokens::Tokenizable,
    futures::Future,
    types::{Address, Bytes, H256, U256},
};

use node_runtime::{bridge, Event};
use parity_codec::Decode;
use primitives;
use substrate_api_client::{hexstr_to_vec, Api};
use system;

use std::sync::mpsc;
use std::thread;

use crate::config;
use crate::ethereum_transactions;
use crate::substrate_transactions;

const AMOUNT: u64 = 0;

pub fn start(config: config::Config) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("substrate_event_handler".to_string())
        .spawn(move || {
            let _sentinel = Sentinel::new(config.clone());
            let _ = thread::spawn(move || {
                log::info!("[substrate] starting event_handler");
                let (events_in, events_out) = mpsc::channel();

                let event_subscriber =
                    start_event_subscriber(config.sub_api_url.clone(), events_in);
                let event_handler = start_event_handler(config, events_out);

                let _ = event_subscriber.join();
                let _ = event_handler.join();
            })
            .join();
        })
        .expect("can not start substrate_event_handler")
}

fn start_event_subscriber(
    api_url: String,
    events_in: mpsc::Sender<String>,
) -> thread::JoinHandle<()> {
    let mut sub_api = Api::new(api_url);
    sub_api.init();

    log::info!("[substrate] starting subscriber of event_handler");
    thread::Builder::new()
        .name("event_subscriber".to_string())
        .spawn(move || {
            sub_api.subscribe_events(events_in.clone());
        })
        .expect("can not start event_subscriber")
}

fn start_event_handler(
    config: config::Config,
    events_out: mpsc::Receiver<String>,
) -> thread::JoinHandle<()> {
    log::info!("[substrate] starting handler of event_handler");
    thread::Builder::new()
        .name("event_handler".to_string())
        .spawn(move || {
            let mut sub_api = Api::new(config.sub_api_url.clone());
            sub_api.init();

            let (_eloop, transport) =
                web3::transports::WebSocket::new(&config.eth_api_url).unwrap();
            let web3 = web3::Web3::new(transport);

            let abi = ethabi::Contract::load(
                include_bytes!("../res/EthContract.abi").to_vec().as_slice(),
            )
            .expect("can read ABI");

            for event in events_out {
                log::debug!("[substrate] got event: {:?}", event);

                let unhex = hexstr_to_vec(event);
                let mut er_enc = unhex.as_slice();
                let events = Vec::<system::EventRecord<Event>>::decode(&mut er_enc);

                match events {
                    Some(evts) => {
                        for evr in &evts {
                            log::debug!(
                                "[substrate] decoded: phase {:?} event {:?}",
                                evr.phase,
                                evr.event
                            );
                            match &evr.event {
                                Event::bridge(br) => {
                                    log::info!("[substrate] bridge event: {:?}", br);
                                    match &br {
                                        bridge::RawEvent::RelayMessage(message_id) => {
                                            handle_replay_message(&sub_api, &config, message_id)
                                        }
                                        bridge::RawEvent::ApprovedRelayMessage(
                                            message_id,
                                            from,
                                            to,
                                            amount,
                                        ) => handle_approved_relay_message(
                                            &web3, &abi, &config, message_id, from, to, *amount,
                                        ),
                                        bridge::RawEvent::Burned(
                                            _message_id,
                                            _from,
                                            _to,
                                            _amount,
                                        ) => (),
                                        bridge::RawEvent::Minted(message_id) => {
                                            handle_minted(&web3, &abi, &config, message_id)
                                        }
                                    }
                                }
                                _ => log::debug!(
                                    "[substrate] ignoring unsupported module event: {:?}",
                                    evr.event
                                ),
                            }
                        }
                    }
                    None => log::error!("[substrate] could not decode event record list"),
                }
            }
        })
        .expect("can not start event_handler")
}

fn handle_replay_message(sub_api: &Api, config: &config::Config, message_id: &primitives::H256) {
    substrate_transactions::approve_transfer(
        &sub_api,
        config.sub_validator_mnemonic_phrase.clone(),
        *message_id,
    );
    log::info!("[substrate] called approve_transfer({:?})", message_id);
}

fn handle_approved_relay_message<T>(
    web3: &web3::Web3<T>,
    abi: &ethabi::Contract,
    config: &config::Config,
    message_id: &primitives::H256,
    from: &primitives::sr25519::Public,
    to: &primitives::H160,
    amount: u64,
) where
    T: web3::Transport + Send + 'static,
    T::Out: Send,
{
    let args = (
        message_id.as_fixed_bytes().into_token(),
        H256::from_slice(from.as_slice())
            .as_fixed_bytes()
            .into_token(),
        Address::from(to.as_fixed_bytes()).into_token(),
        U256::from(amount).into_token(),
    );
    let web3 = web3.clone();
    let eth_validator_private_key = config.eth_validator_private_key.clone();
    let eth_contract_address = config.eth_contract_address;
    let eth_gas_price = config.eth_gas_price;
    let eth_gas = config.eth_gas;
    let data =
        ethereum_transactions::build_transaction_data(&abi, "withdrawTransfer", args.clone());
    let fut = web3.eth().transaction_count(config.eth_validator_address, None)
        .and_then(move |nonce| {
            let tx = ethereum_transactions::build(eth_validator_private_key, eth_contract_address, nonce, AMOUNT, eth_gas_price, eth_gas, data);
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
}

fn handle_minted<T>(
    web3: &web3::Web3<T>,
    abi: &ethabi::Contract,
    config: &config::Config,
    message_id: &primitives::H256,
) where
    T: web3::Transport + Send + 'static,
    T::Out: Send,
{
    let args = (H256::from(message_id.as_fixed_bytes()).into_token(),);

    let web3 = web3.clone();
    let eth_validator_private_key = config.eth_validator_private_key.clone();
    let eth_contract_address = config.eth_contract_address;
    let eth_gas_price = config.eth_gas_price;
    let eth_gas = config.eth_gas;
    let data = ethereum_transactions::build_transaction_data(&abi, "confirmTransfer", args.clone());
    let fut = web3.eth().transaction_count(config.eth_validator_address, None)
        .and_then(move |nonce| {
            let tx = ethereum_transactions::build(eth_validator_private_key, eth_contract_address, nonce, AMOUNT, eth_gas_price, eth_gas, data);
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
}

struct Sentinel {
    config: config::Config,
}

impl Sentinel {
    fn new(config: config::Config) -> Self {
        Sentinel { config }
    }
}

impl Drop for Sentinel {
    fn drop(&mut self) {
        start(self.config.clone());
    }
}

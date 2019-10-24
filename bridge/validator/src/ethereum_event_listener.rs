use ethabi::{self, ParamType};
use futures::stream::Stream;
use log;
use web3::{
    futures::Future,
    types::{H256, Filter, FilterBuilder}
};

use std::{
    thread,
    sync::{Arc, Mutex, mpsc::Sender}
};

use crate::config::Config;
use crate::controller::Events;

struct EventListener {
    config: Config,
    controller_tx: Arc<Mutex<Sender<Events>>>
}

pub fn spawn(config: Config, controller_tx: Sender<Events>) -> thread::JoinHandle<()> {
    thread::Builder::new()
    .name("ethereum_event_listener".to_string())
    .spawn(move || {
        let event_listener = EventListener::new(config, controller_tx);
        event_listener.start();
    })
    .expect("can not started ethereum_event_listener")
}

impl EventListener {
    fn new(config: Config, controller_tx: Sender<Events>) -> Self {
        let controller_tx = Arc::new(Mutex::new(controller_tx));
        EventListener {
            config,
            controller_tx
        }
    }

    fn start(&self) {
        let (_eloop, transport) = web3::transports::WebSocket::new(&self.config.eth_api_url).unwrap();
        let web3 = web3::Web3::new(transport);

        let controller_tx = self.controller_tx.clone();
        let config = self.config.clone();

        let fut = web3
            .eth_subscribe()
            .subscribe_logs(build_filter(&config))
            .then(move |sub| {
                sub.unwrap().for_each(move |log| {
                    log::debug!("[ethereum] got log: {:?}", log);
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
                        (true, _, _) => {
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
                                if params.len() >= 4 {
                                    let event = Events::EthRelayMessage(
                                        H256::from_slice(&params[0].clone().to_fixed_bytes().expect("can not parse message_id")),
                                        params[1].clone().to_address().expect("can not parse eth_address"),
                                        H256::from_slice(&params[2].clone().to_fixed_bytes().expect("can not parse sub_address")),
                                        params[3].clone().to_uint().expect("can not parse amount")
                                    );
                                    controller_tx.lock().unwrap().send(event).expect("can not send event");
                                }
                            }
                            Ok(())
                        },
                        (_, true, _) => {
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
                                if params.len() >= 4 {
                                   let event = Events::EthApprovedRelayMessage(
                                        H256::from_slice(&params[0].clone().to_fixed_bytes().expect("can not parse message_id")),
                                        params[1].clone().to_address().expect("can not parse eth_address"),
                                        H256::from_slice(&params[2].clone().to_fixed_bytes().expect("can not parse sub_address")),
                                        params[3].clone().to_uint().expect("can not parse amount")
                                    );
                                    controller_tx.lock().unwrap().send(event).expect("can not send event");
                                }
                            }
                            Ok(())
                        },
                        (_, _, true) => {
                            let result = ethabi::decode(
                                &[
                                    ParamType::FixedBytes(32)
                                ],
                                &log.data.0,
                            );
                            if let Ok(params) = result {
                                if !params.is_empty() {
                                    let event = Events::EthWithdrawMessage(
                                        H256::from_slice(&params[0].clone().to_fixed_bytes().expect("can not parse message_id"))
                                    );
                                    controller_tx.lock().unwrap().send(event).expect("can not send event");
                                }
                            }
                            Ok(())
                        },
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
}

fn build_filter(config: &Config) -> Filter {
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

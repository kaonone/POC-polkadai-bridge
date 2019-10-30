use futures::future::{lazy, poll_fn};
use log;
use primitives;
use substrate_api_client::Api;
use tokio::runtime::{Runtime, TaskExecutor};
use tokio_threadpool::blocking;
use web3::{
    futures::Future,
    types::{Bytes, H160, H256, U256},
};

use std::{
    sync::{mpsc::Receiver, Arc},
    thread,
};

use crate::config::Config;
use crate::controller::Event;
use crate::ethereum_transactions;
use crate::substrate_transactions;

const AMOUNT: u64 = 0;

#[derive(Debug)]
struct Executor {
    config: Config,
    executor_rx: Receiver<Event>,
}

pub fn spawn(config: Config, executor_rx: Receiver<Event>) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("executor".to_string())
        .spawn(move || {
            let executor = Executor::new(config, executor_rx);
            executor.start()
        })
        .expect("can not started executor")
}

impl Executor {
    fn new(config: Config, executor_rx: Receiver<Event>) -> Self {
        Executor {
            config,
            executor_rx,
        }
    }

    fn start(&self) {
        let runtime = Runtime::new().expect("can not create tokio runtime");

        let mut sub_api = Api::new(self.config.sub_api_url.clone());
        sub_api.init();
        let sub_api = Arc::new(sub_api);

        let (_eloop, transport) =
            web3::transports::WebSocket::new(&self.config.eth_api_url).unwrap();
        let web3 = web3::Web3::new(transport);

        let contact_abi = include_bytes!("../res/EthContract.abi");
        let abi = ethabi::Contract::load(contact_abi.to_vec().as_slice()).expect("can read ABI");

        let abi = Arc::new(abi);
        let web3 = Arc::new(web3);

        self.executor_rx.iter().for_each(|event| {
            log::info!("received event: {:?}", event);
            match event {
                Event::EthRelayMessage(
                    message_id,
                    eth_address,
                    sub_address,
                    amount,
                    _block_number,
                ) => handle_eth_relay_message(
                    &self.config,
                    runtime.executor(),
                    web3.clone(),
                    abi.clone(),
                    message_id,
                    eth_address,
                    sub_address,
                    amount,
                ),
                Event::EthApprovedRelayMessage(
                    message_id,
                    eth_address,
                    sub_address,
                    amount,
                    _block_number,
                ) => handle_eth_approved_relay_message(
                    &self.config,
                    runtime.executor(),
                    sub_api.clone(),
                    message_id,
                    eth_address,
                    sub_address,
                    amount,
                ),
                Event::EthRevertMessage(message_id, _eth_address, _amount, _block_number) => {
                    handle_eth_revert_message(
                        &self.config,
                        runtime.executor(),
                        sub_api.clone(),
                        message_id,
                    )
                }
                Event::EthWithdrawMessage(message_id, _block_number) => {
                    handle_eth_withdraw_message(
                        &self.config,
                        runtime.executor(),
                        sub_api.clone(),
                        message_id,
                    )
                }
                Event::SubRelayMessage(message_id, _block_number) => handle_sub_relay_message(
                    &self.config,
                    runtime.executor(),
                    sub_api.clone(),
                    message_id,
                ),
                Event::SubApprovedRelayMessage(
                    message_id,
                    sub_address,
                    eth_address,
                    amount,
                    _block_number,
                ) => handle_sub_approved_relay_message(
                    &self.config,
                    runtime.executor(),
                    web3.clone(),
                    abi.clone(),
                    message_id,
                    sub_address,
                    eth_address,
                    amount,
                ),
                Event::SubBurnedMessage(
                    _message_id,
                    _sub_address,
                    _eth_address,
                    _amount,
                    _block_number,
                ) => (),
                Event::SubMintedMessage(message_id, _block_number) => handle_sub_minted_message(
                    &self.config,
                    runtime.executor(),
                    web3.clone(),
                    abi.clone(),
                    message_id,
                ),
            }
        })
    }
}

fn handle_eth_relay_message<T>(
    config: &Config,
    task_executor: TaskExecutor,
    web3: Arc<web3::Web3<T>>,
    abi: Arc<ethabi::Contract>,
    message_id: H256,
    eth_address: H160,
    sub_address: H256,
    amount: U256,
) where
    T: web3::Transport + Send + Sync + 'static,
    T::Out: Send,
{
    let args = (message_id, eth_address, sub_address, amount);
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
                                        message_id, eth_address, sub_address, amount, nonce, tx_res);
                        },
                        Err(err) => {
                            log::warn!("[ethereum] can not send approveTransfer({:?}, {:?}, {:?}, {:?}), nonce: {:?}, reason: {:?}",
                                        message_id, eth_address, sub_address, amount, nonce, err);
                        }
                    }
                    Ok(())
                })

        })
        .map_err(|e| log::warn!("can not get nonce: {:?}", e));
    task_executor.spawn(fut);
}

fn handle_eth_approved_relay_message(
    config: &Config,
    task_executor: TaskExecutor,
    sub_api: Arc<Api>,
    message_id: H256,
    eth_address: H160,
    sub_address: H256,
    amount: U256,
) {
    let message_id = primitives::H256::from_slice(&message_id.to_fixed_bytes());
    let eth_address = primitives::H160::from_slice(&eth_address.to_fixed_bytes());
    let sub_address = primitives::sr25519::Public::from_slice(&sub_address.to_fixed_bytes());
    let amount = amount.low_u128();
    let sub_validator_mnemonic_phrase = config.sub_validator_mnemonic_phrase.clone();

    task_executor.spawn(lazy(move || {
        poll_fn(move || {
            blocking(|| {
                substrate_transactions::mint(
                    &sub_api.clone(),
                    sub_validator_mnemonic_phrase.clone(),
                    message_id,
                    eth_address,
                    sub_address.clone(),
                    amount,
                );
                log::info!(
                    "[substrate] called multi_signed_mint({:?}, {:?}, {:?}, {:?})",
                    message_id,
                    eth_address,
                    sub_address,
                    amount
                );
            })
            .map_err(|_| panic!("the threadpool shut down"))
        })
    }));
}

fn handle_eth_revert_message(
    config: &Config,
    task_executor: TaskExecutor,
    sub_api: Arc<Api>,
    message_id: H256,
) {
    let message_id = primitives::H256::from_slice(&message_id.to_fixed_bytes());
    let sub_validator_mnemonic_phrase = config.sub_validator_mnemonic_phrase.clone();

    task_executor.spawn(lazy(move || {
        poll_fn(move || {
            blocking(|| {
                substrate_transactions::cancel_transfer(
                    &sub_api.clone(),
                    sub_validator_mnemonic_phrase.clone(),
                    message_id,
                );
                log::info!("[substrate] called cancel_transfer({:?})", message_id);
            })
            .map_err(|_| panic!("the threadpool shut down"))
        })
    }));
}

fn handle_eth_withdraw_message(
    config: &Config,
    task_executor: TaskExecutor,
    sub_api: Arc<Api>,
    message_id: H256,
) {
    let message_id = primitives::H256::from_slice(&message_id.to_fixed_bytes());
    let sub_validator_mnemonic_phrase = config.sub_validator_mnemonic_phrase.clone();

    task_executor.spawn(lazy(move || {
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

fn handle_sub_relay_message(
    config: &Config,
    task_executor: TaskExecutor,
    sub_api: Arc<Api>,
    message_id: H256,
) {
    let message_id = primitives::H256::from_slice(&message_id.to_fixed_bytes());
    let sub_validator_mnemonic_phrase = config.sub_validator_mnemonic_phrase.clone();

    task_executor.spawn(lazy(move || {
        poll_fn(move || {
            blocking(|| {
                substrate_transactions::approve_transfer(
                    &sub_api,
                    sub_validator_mnemonic_phrase.clone(),
                    message_id,
                );
                log::info!("[substrate] called approve_transfer({:?})", message_id);
            })
            .map_err(|_| panic!("the threadpool shut down"))
        })
    }));
}

fn handle_sub_approved_relay_message<T>(
    config: &Config,
    task_executor: TaskExecutor,
    web3: Arc<web3::Web3<T>>,
    abi: Arc<ethabi::Contract>,
    message_id: H256,
    sub_address: H256,
    eth_address: H160,
    amount: U256,
) where
    T: web3::Transport + Send + Sync + 'static,
    T::Out: Send,
{
    let args = (message_id, sub_address, eth_address, amount);
    let eth_validator_private_key = config.eth_validator_private_key.clone();
    let eth_contract_address = config.eth_contract_address;
    let eth_gas_price = config.eth_gas_price;
    let eth_gas = config.eth_gas;
    let data = ethereum_transactions::build_transaction_data(&abi, "withdrawTransfer", args);
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
    task_executor.spawn(fut);
}

fn handle_sub_minted_message<T>(
    config: &Config,
    task_executor: TaskExecutor,
    web3: Arc<web3::Web3<T>>,
    abi: Arc<ethabi::Contract>,
    message_id: H256,
) where
    T: web3::Transport + Send + Sync + 'static,
    T::Out: Send,
{
    let args = (message_id,);
    let eth_validator_private_key = config.eth_validator_private_key.clone();
    let eth_contract_address = config.eth_contract_address;
    let eth_gas_price = config.eth_gas_price;
    let eth_gas = config.eth_gas;
    let data = ethereum_transactions::build_transaction_data(&abi, "confirmTransfer", args);
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
    task_executor.spawn(fut);
}

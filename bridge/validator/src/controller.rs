use web3::types::{H160, H256, U256};

use log;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::config::Config;
use crate::controller_storage::ControllerStorage;

type MessageId = H256;
type EthAddress = H160;
type SubAddress = H256;
type Amount = U256;
type BlockNumber = u128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    EthRelayMessage(MessageId, EthAddress, SubAddress, Amount, BlockNumber),
    EthApprovedRelayMessage(MessageId, EthAddress, SubAddress, Amount, BlockNumber),
    EthRevertMessage(MessageId, EthAddress, Amount, BlockNumber),
    EthWithdrawMessage(MessageId, BlockNumber),
    SubRelayMessage(MessageId, BlockNumber),
    SubApprovedRelayMessage(MessageId, SubAddress, EthAddress, Amount, BlockNumber),
    SubBurnedMessage(MessageId, SubAddress, EthAddress, Amount, BlockNumber),
    SubMintedMessage(MessageId, BlockNumber),
}

#[derive(Debug, Clone)]
enum Status {
    NotReady,
    Active,
    Paused,
    Stopped,
}

#[derive(Debug)]
struct Controller {
    config: Config,
    status: Status,
    controller_rx: Receiver<Event>,
    executor_tx: Sender<Event>,
    storage: ControllerStorage,
}

pub fn spawn(
    config: Config,
    controller_rx: Receiver<Event>,
    executor_tx: Sender<Event>,
) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("controller".to_string())
        .spawn(move || {
            let mut controller = Controller::new(config, controller_rx, executor_tx);
            controller.start();
        })
        .expect("can not started controller")
}

impl Event {
    pub fn message_id(&self) -> &H256 {
        match self {
            Self::EthRelayMessage(message_id, _, _, _, _) => message_id,
            Self::EthApprovedRelayMessage(message_id, _, _, _, _) => message_id,
            Self::EthRevertMessage(message_id, _, _, _) => message_id,
            Self::EthWithdrawMessage(message_id, _) => message_id,
            Self::SubRelayMessage(message_id, _) => message_id,
            Self::SubApprovedRelayMessage(message_id, _, _, _, _) => message_id,
            Self::SubBurnedMessage(message_id, _, _, _, _) => message_id,
            Self::SubMintedMessage(message_id, _) => message_id,
        }
    }

    pub fn block_number(&self) -> u128 {
        match self {
            Self::EthRelayMessage(_, _, _, _, block_number) => *block_number,
            Self::EthApprovedRelayMessage(_, _, _, _, block_number) => *block_number,
            Self::EthRevertMessage(_, _, _, block_number) => *block_number,
            Self::EthWithdrawMessage(_, block_number) => *block_number,
            Self::SubRelayMessage(_, block_number) => *block_number,
            Self::SubApprovedRelayMessage(_, _, _, _, block_number) => *block_number,
            Self::SubBurnedMessage(_, _, _, _, block_number) => *block_number,
            Self::SubMintedMessage(_, block_number) => *block_number,
        }
    }
}

impl Controller {
    fn new(config: Config, controller_rx: Receiver<Event>, executor_tx: Sender<Event>) -> Self {
        Controller {
            config,
            status: Status::NotReady,
            controller_rx,
            executor_tx,
            storage: ControllerStorage::new(),
        }
    }

    fn start(&mut self) {
        log::info!("current status: {:?}", self.status);
        self.update_status(Status::Active);
        let storage = &mut self.storage;
        let controller_rx = &self.controller_rx;
        let status = &self.status;
        let executor_tx = &self.executor_tx;
        controller_rx
            .iter()
            .for_each(|event| match storage.put_event(&event) {
                Ok(()) => {
                    log::info!("received event: {:?}", event);
                    match status {
                        Status::Active => {
                            storage.iter_events_queue().cloned().for_each(|event| {
                                executor_tx.send(event).expect("can not sent event")
                            });
                            storage.clear_events_queue();
                            executor_tx.send(event).expect("can not sent event")
                        }
                        Status::NotReady | Status::Paused | Status::Stopped => {
                            storage.put_event_to_queue(event)
                        }
                    }
                }
                Err(e) => log::debug!("controller storage error: {:?}", e),
            })
    }

    fn update_status(&mut self, status: Status) {
        self.status = status;
        log::info!("current status: {:?}", self.status);
    }
}

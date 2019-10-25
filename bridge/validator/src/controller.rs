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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Events {
    EthRelayMessage(MessageId, EthAddress, SubAddress, Amount),
    EthApprovedRelayMessage(MessageId, EthAddress, SubAddress, Amount),
    EthWithdrawMessage(MessageId),
    SubRelayMessage(MessageId),
    SubApprovedRelayMessage(MessageId, SubAddress, EthAddress, Amount),
    SubBurnedMessage(MessageId, SubAddress, EthAddress, Amount),
    SubMintedMessage(MessageId),
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
    controller_rx: Receiver<Events>,
    executor_tx: Sender<Events>,
    storage: ControllerStorage,
}

pub fn spawn(
    config: Config,
    controller_rx: Receiver<Events>,
    executor_tx: Sender<Events>,
) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("controller".to_string())
        .spawn(move || {
            let mut controller = Controller::new(config, controller_rx, executor_tx);
            controller.start();
        })
        .expect("can not started controller")
}

impl Events {
    pub fn message_id(&self) -> &H256 {
        match self {
            Self::EthRelayMessage(message_id, _, _, _) => message_id,
            Self::EthApprovedRelayMessage(message_id, _, _, _) => message_id,
            Self::EthWithdrawMessage(message_id) => message_id,
            Self::SubRelayMessage(message_id) => message_id,
            Self::SubApprovedRelayMessage(message_id, _, _, _) => message_id,
            Self::SubBurnedMessage(message_id, _, _, _) => message_id,
            Self::SubMintedMessage(message_id) => message_id,
        }
    }
}

impl Controller {
    fn new(config: Config, controller_rx: Receiver<Events>, executor_tx: Sender<Events>) -> Self {
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
                        Status::Active => executor_tx.send(event).expect("can not sent event"),
                        Status::NotReady | Status::Paused | Status::Stopped => (),
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

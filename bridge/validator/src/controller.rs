use web3::types::{H160, H256, U256};

use log;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::config::Config;

type MessageId = H256;
type EthAddress = H160;
type SubAddress = H256;
type Amount = U256;

#[derive(Debug, Clone)]
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

impl Controller {
    fn new(config: Config, controller_rx: Receiver<Events>, executor_tx: Sender<Events>) -> Self {
        Controller {
            config,
            status: Status::NotReady,
            controller_rx,
            executor_tx,
        }
    }

    fn start(&mut self) {
        log::info!("current status: {:?}", self.status);
        self.update_status(Status::Active);
        self.controller_rx.iter().for_each(|event| {
            log::info!("received event: {:?}", event);
            match self.status {
                Status::Active => self.executor_tx.send(event).expect("can not sent event"),
                Status::NotReady | Status::Paused | Status::Stopped => (),
            }
        })
    }

    fn update_status(&mut self, status: Status) {
        self.status = status;
        log::info!("current status: {:?}", self.status);
    }
}

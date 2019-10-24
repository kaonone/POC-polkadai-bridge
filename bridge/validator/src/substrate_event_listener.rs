use log;
use web3::types::{H160, H256, U256};

use node_runtime::{bridge, Event, bridge::RawEvent as BridgeEvent};
use parity_codec::Decode;
use primitives;
use substrate_api_client::{hexstr_to_vec, Api};
use system;

use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::config::Config;
use crate::controller::Events;

#[derive(Debug, Clone)]
struct EventListener {
    config: Config,
    events_in: Sender<String>,
}

struct EventHandler {
    controller_tx: Sender<Events>,
    events_out: Receiver<String>
}

pub fn spawn(config: Config, controller_tx: Sender<Events>) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("substrate_event_processor".to_string())
        .spawn(move || {
            let (events_in, events_out) = channel();
            let event_listener = thread::Builder::new()
                .name("substrate_event_listener".to_string())
                .spawn(move || {
                    let event_listener = EventListener::new(config, events_in);
                    event_listener.start();
                })
                .expect("can not started substrate_event_listener");

            let event_handler = thread::Builder::new()
                .name("substrate_event_handler".to_string())
                .spawn(move || {
                    let event_handler = EventHandler::new(controller_tx, events_out);
                    event_handler.start();
                })
                .expect("can not started substrate_event_handler");

            let _ = event_listener.join();
            let _ = event_handler.join();
    })
    .expect("can not started substrate_event_processor")
}

impl EventListener {
    fn new(config: Config, events_in: Sender<String>) -> Self {
        EventListener {
            config,
            events_in
        }
    }

    fn start(&self) {
        let mut sub_api = Api::new(self.config.sub_api_url.clone());
        sub_api.init();
        sub_api.subscribe_events(self.events_in.clone());
    }
}

impl EventHandler {
    fn new(controller_tx: Sender<Events>, events_out: Receiver<String>) -> Self {
        EventHandler {
            controller_tx,
            events_out
        }
    }

    fn start(&self) {
        self.events_out.iter().for_each(|event| {
            log::debug!("[substrate] got event: {:?}", event);

            let unhex = hexstr_to_vec(event);
            let mut er_enc = unhex.as_slice();
            let events = Vec::<system::EventRecord<Event>>::decode(&mut er_enc);

            match events {
                Some(evts) => {
                    for evr in &evts {
                        log::debug!("[substrate] decoded: phase {:?} event {:?}", evr.phase, evr.event);
                        match &evr.event {
                            Event::bridge(bridge_event) => self.handle_bridge_event(bridge_event),
                            _ => log::debug!("[substrate] ignoring unsupported module event: {:?}", evr.event),
                        }
                    }
                }
                None => log::error!("[substrate] could not decode event record list"),
            }
        })
    }

    fn handle_bridge_event(&self, event: &BridgeEvent<primitives::sr25519::Public, primitives::H256>) {
        log::info!("[substrate] bridge event: {:?}", event);
        match &event {
            bridge::RawEvent::RelayMessage(message_id) => {
                let event = Events::SubRelayMessage(
                    H256::from_slice(message_id.as_bytes())
                );
                self.controller_tx.send(event).unwrap();
            },
            bridge::RawEvent::ApprovedRelayMessage(message_id, from, to, amount) => {
                let event = Events::SubApprovedRelayMessage(
                    H256::from_slice(message_id.as_bytes()),
                    H256::from_slice(from.as_slice()),
                    H160::from_slice(to.as_bytes()),
                    U256::from(*amount)
                );
                self.controller_tx.send(event).unwrap();
            },
            bridge::RawEvent::BurnedMessage(message_id, from, to, amount) => {
                let event = Events::SubBurnedMessage(
                    H256::from_slice(message_id.as_bytes()),
                    H256::from_slice(from.as_slice()),
                    H160::from_slice(to.as_bytes()),
                    U256::from(*amount)
                );
                self.controller_tx.send(event).unwrap();
            }
            bridge::RawEvent::MintedMessage(message_id) => {
                let event = Events::SubMintedMessage(
                    H256::from_slice(message_id.as_bytes())
                );
                self.controller_tx.send(event).unwrap();
            }
        }
    }
}

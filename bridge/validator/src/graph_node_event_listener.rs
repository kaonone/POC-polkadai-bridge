use graphql_client::{GraphQLQuery, Response};
use reqwest;
use rustc_hex::FromHex;
use web3::types::{H160, H256, U256};

use std::{sync::mpsc::Sender, thread, time::Duration};

use crate::config::Config;
use crate::controller::Event;

struct EventListener {
    config: Config,
    controller_tx: Sender<Event>,
    max_block_number: u64,
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "res/graph_node_schema.graphql",
    query_path = "res/graph_node_max_block_number.graphql",
    response_derives = "Debug"
)]
struct MaxBlockNumber;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "res/graph_node_schema.graphql",
    query_path = "res/graph_node_all_messages.graphql",
    response_derives = "Debug,Clone"
)]
struct AllMessages;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "res/graph_node_schema.graphql",
    query_path = "res/graph_node_messages_by_status.graphql",
    response_derives = "Debug,Clone"
)]
struct MessagesByStatus;

pub fn spawn(config: Config, controller_tx: Sender<Event>) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("graph_node_event_listener".to_string())
        .spawn(move || {
            let mut event_listener = EventListener::new(config, controller_tx);
            event_listener.start();
        })
        .expect("can not started graph_node_listener")
}

impl EventListener {
    fn new(config: Config, controller_tx: Sender<Event>) -> Self {
        EventListener {
            config,
            controller_tx,
            max_block_number: 0,
        }
    }

    fn start(&mut self) {
        let _ = self.get_max_block_number().and_then(|block_number| {
            self.update_max_block_number(block_number);
            Ok(())
        });
        self.send_unfinalized_transactions();

        loop {
            let _ = self.get_all_messages().and_then(|events| {
                self.send_events(events);
                Ok(())
            });
            thread::sleep(Duration::from_millis(1000));
        }
    }

    fn send_events(&self, events: Vec<Event>) {
        events
            .iter()
            .cloned()
            .for_each(|event| self.controller_tx.send(event).expect("can not send event"));
    }

    fn send_unfinalized_transactions(&self) {
        const UNFINALIZED_STATUSES: [messages_by_status::Status; 4] = [
            messages_by_status::Status::PENDING,
            messages_by_status::Status::WITHDRAW,
            messages_by_status::Status::APPROVED,
            messages_by_status::Status::CANCELED,
        ];

        let mut events: Vec<_> = UNFINALIZED_STATUSES
            .iter()
            .cloned()
            .map(|status| {
                self.get_messages_by_status(status)
                    .unwrap_or_else(|_| vec![])
            })
            .flatten()
            .collect();

        events.sort_by(|a, b| a.block_number().cmp(&b.block_number()));
        self.send_events(events);
    }

    fn get_max_block_number(&self) -> Result<u64, reqwest::Error> {
        let request_body = MaxBlockNumber::build_query(max_block_number::Variables {
            block_number: self.max_block_number as i64,
        });
        let client = reqwest::Client::new();
        let mut res = client
            .post(&self.config.graph_node_api_url)
            .json(&request_body)
            .send()?;
        let response_body: Response<max_block_number::ResponseData> = res.json()?;
        let messages = response_body
            .data
            .expect("can not get response_data")
            .messages;
        if messages.is_empty() {
            Ok(self.max_block_number)
        } else {
            Ok(messages[0]
                .eth_block_number
                .parse()
                .expect("can not parse eth_block_number"))
        }
    }

    fn get_all_messages(&mut self) -> Result<Vec<Event>, reqwest::Error> {
        let request_body = AllMessages::build_query(all_messages::Variables {
            block_number: self.max_block_number as i64,
        });
        let client = reqwest::Client::new();
        let mut res = client
            .post(&self.config.graph_node_api_url)
            .json(&request_body)
            .send()?;
        let response_body: Response<all_messages::ResponseData> = res.json()?;
        let messages = response_body
            .data
            .expect("can not get response_data")
            .messages;

        messages
            .iter()
            .map(|message| {
                message
                    .eth_block_number
                    .parse()
                    .expect("can not parase eth_block_number")
            })
            .max()
            .and_then(|eth_block_number| {
                self.update_max_block_number(eth_block_number);
                Some(eth_block_number)
            });

        Ok(messages.iter().map(Into::into).collect())
    }

    fn get_messages_by_status(
        &self,
        status: messages_by_status::Status,
    ) -> Result<Vec<Event>, reqwest::Error> {
        log::info!("getting unfinalized transactions, status={:?}", status);
        let request_body = MessagesByStatus::build_query(messages_by_status::Variables {
            eth_block_number: 0,
            status: status.clone(),
        });
        let client = reqwest::Client::new();
        let mut res = client
            .post(&self.config.graph_node_api_url)
            .json(&request_body)
            .send()?;
        let response_body: Response<messages_by_status::ResponseData> = res.json()?;
        let messages = response_body
            .data
            .expect("can not get response_data")
            .messages;

        log::info!(
            "got {} unfinalized transactions, status={:?}",
            messages.len(),
            status
        );
        Ok(messages.iter().map(Into::into).collect())
    }

    fn update_max_block_number(&mut self, block_number: u64) {
        self.max_block_number = block_number;
        log::debug!("max_block_number: {:?}", self.max_block_number);
    }
}

impl From<&all_messages::AllMessagesMessages> for Event {
    fn from(message: &all_messages::AllMessagesMessages) -> Event {
        match (&message.status, &message.direction) {
            (all_messages::Status::PENDING, all_messages::Direction::ETH2SUB) => {
                Event::EthRelayMessage(
                    parse_h256(&message.id),
                    parse_h160(&message.eth_address),
                    parse_h256(&message.sub_address),
                    parse_u256(&message.amount),
                    parse_u128(&message.eth_block_number),
                )
            }
            (all_messages::Status::APPROVED, all_messages::Direction::ETH2SUB) => {
                Event::EthApprovedRelayMessage(
                    parse_h256(&message.id),
                    parse_h160(&message.eth_address),
                    parse_h256(&message.sub_address),
                    parse_u256(&message.amount),
                    parse_u128(&message.eth_block_number),
                )
            }
            (all_messages::Status::CANCELED, all_messages::Direction::ETH2SUB) => {
                Event::EthRevertMessage(
                    parse_h256(&message.id),
                    parse_h160(&message.eth_address),
                    parse_u256(&message.amount),
                    parse_u128(&message.eth_block_number),
                )
            }
            (all_messages::Status::WITHDRAW, all_messages::Direction::SUB2ETH) => {
                Event::EthWithdrawMessage(
                    parse_h256(&message.id),
                    parse_u128(&message.eth_block_number),
                )
            }

            (_, _) => Event::EthApprovedRelayMessage(
                parse_h256(&message.id),
                parse_h160(&message.eth_address),
                parse_h256(&message.sub_address),
                parse_u256(&message.amount),
                parse_u128(&message.eth_block_number),
            ),
        }
    }
}

impl From<&messages_by_status::MessagesByStatusMessages> for Event {
    fn from(message: &messages_by_status::MessagesByStatusMessages) -> Self {
        match (&message.status, &message.direction) {
            (messages_by_status::Status::PENDING, messages_by_status::Direction::ETH2SUB) => {
                Event::EthRelayMessage(
                    parse_h256(&message.id),
                    parse_h160(&message.eth_address),
                    parse_h256(&message.sub_address),
                    parse_u256(&message.amount),
                    parse_u128(&message.eth_block_number),
                )
            }
            (messages_by_status::Status::APPROVED, messages_by_status::Direction::ETH2SUB) => {
                Event::EthApprovedRelayMessage(
                    parse_h256(&message.id),
                    parse_h160(&message.eth_address),
                    parse_h256(&message.sub_address),
                    parse_u256(&message.amount),
                    parse_u128(&message.eth_block_number),
                )
            }
            (messages_by_status::Status::CANCELED, messages_by_status::Direction::ETH2SUB) => {
                Event::EthRevertMessage(
                    parse_h256(&message.id),
                    parse_h160(&message.eth_address),
                    parse_u256(&message.amount),
                    parse_u128(&message.eth_block_number),
                )
            }
            (messages_by_status::Status::WITHDRAW, messages_by_status::Direction::SUB2ETH) => {
                Event::EthWithdrawMessage(
                    parse_h256(&message.id),
                    parse_u128(&message.eth_block_number),
                )
            }

            (_, _) => Event::EthApprovedRelayMessage(
                parse_h256(&message.id),
                parse_h160(&message.eth_address),
                parse_h256(&message.sub_address),
                parse_u256(&message.amount),
                parse_u128(&message.eth_block_number),
            ),
        }
    }
}

fn parse_h256(hash: &str) -> H256 {
    H256::from_slice(&hash[2..].from_hex::<Vec<_>>().expect("can not parse H256"))
}

fn parse_h160(hash: &str) -> H160 {
    H160::from_slice(&hash[2..].from_hex::<Vec<_>>().expect("can not parse H160"))
}

fn parse_u256(maybe_u256: &str) -> U256 {
    maybe_u256.parse().expect("can not parse U256")
}

fn parse_u128(maybe_u128: &str) -> u128 {
    maybe_u128.parse().expect("can not parse u128")
}

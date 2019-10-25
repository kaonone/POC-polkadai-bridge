use graphql_client::{GraphQLQuery, Response};
use reqwest;
use rustc_hex::FromHex;
use web3::types::{H160, H256};

use std::{sync::mpsc::Sender, thread, time::Duration};

use crate::config::Config;
use crate::controller::Events;

struct EventListener {
    config: Config,
    controller_tx: Sender<Events>,
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

pub fn spawn(config: Config, controller_tx: Sender<Events>) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("graph_node_event_listener".to_string())
        .spawn(move || {
            let mut event_listener = EventListener::new(config, controller_tx);
            event_listener.start();
        })
        .expect("can not started graph_node_listener")
}

impl EventListener {
    fn new(config: Config, controller_tx: Sender<Events>) -> Self {
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
        loop {
            let _ = self.get_all_messages().and_then(|messages| {
                messages.iter().cloned().for_each(|message| {
                    self.controller_tx
                        .send(message)
                        .expect("can not send event")
                });
                Ok(())
            });
            thread::sleep(Duration::from_millis(1000));
        }
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

    fn get_all_messages(&mut self) -> Result<Vec<Events>, reqwest::Error> {
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

        Ok(messages.iter().map(convert_message).collect())
    }

    fn update_max_block_number(&mut self, block_number: u64) {
        self.max_block_number = block_number;
        log::debug!("max_block_number: {:?}", self.max_block_number);
    }
}

fn convert_message(message: &all_messages::AllMessagesMessages) -> Events {
    match (&message.status, &message.direction) {
        (all_messages::Status::PENDING, all_messages::Direction::ETH2SUB) => {
            Events::EthRelayMessage(
                parse_h256(&message.id),
                parse_h160(&message.eth_address),
                parse_h256(&message.sub_address),
                message.amount.parse().expect("can not parse amount"),
            )
        }
        (all_messages::Status::APPROVED, all_messages::Direction::ETH2SUB) => {
            Events::EthApprovedRelayMessage(
                parse_h256(&message.id),
                parse_h160(&message.eth_address),
                parse_h256(&message.sub_address),
                message.amount.parse().expect("can not parse amount"),
            )
        }
        (all_messages::Status::WITHDRAW, all_messages::Direction::SUB2ETH) => {
            Events::EthWithdrawMessage(parse_h256(&message.id))
        }

        (_, _) => Events::EthApprovedRelayMessage(
            parse_h256(&message.id),
            parse_h160(&message.eth_address),
            parse_h256(&message.sub_address),
            message.amount.parse().expect("can not parse amount"),
        ),
    }
}

fn parse_h256(hash: &str) -> H256 {
    H256::from_slice(&hash[2..].from_hex::<Vec<_>>().expect("can not parse H256"))
}

fn parse_h160(hash: &str) -> H160 {
    H160::from_slice(&hash[2..].from_hex::<Vec<_>>().expect("can not parse H160"))
}

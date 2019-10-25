use dotenv::dotenv;
use env_logger;

use std::sync::mpsc::channel;

mod config;
mod controller;
mod ethereum_event_listener;
mod ethereum_transactions;
mod executor;
mod substrate_event_listener;
mod substrate_transactions;

fn main() {
    env_logger::init();
    dotenv().ok();

    let config = config::Config::load().expect("can not load config");

    let (controller_tx, controller_rx) = channel();
    let (executor_tx, executor_rx) = channel();

    let controller_thread = controller::spawn(config.clone(), controller_rx, executor_tx);
    let executor_thread = executor::spawn(config.clone(), executor_rx);
    let ethereum_event_listener_thread =
        ethereum_event_listener::spawn(config.clone(), controller_tx.clone());
    let substrate_event_listener_thread =
        substrate_event_listener::spawn(config.clone(), controller_tx.clone());

    let _ = controller_thread.join();
    let _ = executor_thread.join();
    let _ = ethereum_event_listener_thread.join();
    let _ = substrate_event_listener_thread.join();
}

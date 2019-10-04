use dotenv::dotenv;
use env_logger;

mod config;
mod ethereum_event_handler;
mod ethereum_transactions;
mod substrate_event_handler;
mod substrate_transactions;

fn main() {
    env_logger::init();
    dotenv().ok();

    let config = config::Config::load().expect("can not load config");

    log::info!("[ethereum] api url: {:?}", config.eth_api_url);
    log::info!(
        "[ethereum] validator address: {:?}",
        config.eth_validator_address
    );
    log::info!(
        "[ethereum] contract address: {:?}",
        config.eth_contract_address
    );
    log::info!(
        "[ethereum] hash of RelayMessage: {:?}",
        config.eth_relay_message_hash
    );
    log::info!(
        "[ethereum] hash of ApprovedRelayMessage: {:?}",
        config.eth_approved_relay_message_hash
    );
    log::info!("[substrate] api url: {:?}", config.sub_api_url);

    let _substrate_event_handler = substrate_event_handler::start(config.clone());
    ethereum_event_handler::start(config);
}

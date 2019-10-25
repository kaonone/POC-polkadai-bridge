use erc20_substrate_bridge_runtime::{
    AccountId, BalancesConfig, BridgeConfig, ConsensusConfig, ContractConfig, CouncilVotingConfig,
    DemocracyConfig, GenesisConfig, GrandpaConfig, IndicesConfig, Perbill, Permill, Schedule,
    SessionConfig, StakerStatus, StakingConfig, SudoConfig, TimestampConfig, TreasuryConfig,
};
use primitives::{crypto::UncheckedInto, ed25519, sr25519, Pair};
use substrate_service;

use ed25519::Public as AuthorityId;

use telemetry::TelemetryEndpoints;

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = substrate_service::ChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
    /// Whatever the current runtime is, with just Alice as an auth.
    Development,
    /// Whatever the current runtime is, with simple Alice/Bob auths.
    LocalTestnet,
    Akropolis,
    AkropolisStaging,
}

fn authority_key(s: &str) -> AuthorityId {
    ed25519::Pair::from_string(&format!("//{}", s), None)
        .expect("static values are valid; qed")
        .public()
}

fn account_key(s: &str) -> AccountId {
    sr25519::Pair::from_string(&format!("//{}", s), None)
        .expect("static values are valid; qed")
        .public()
}

impl Alternative {
    /// Get an actual chain config from one of the alternatives.
    pub(crate) fn load(self) -> Result<ChainSpec, String> {
        Ok(match self {
            Alternative::Development => ChainSpec::from_genesis(
                "Development",
                "dev",
                || {
                    testnet_genesis(
                        vec![authority_key("Alice")],
                        vec![account_key("Alice")],
                        account_key("Alice"),
                    )
                },
                vec![],
                None,
                None,
                None,
                None,
            ),
            Alternative::LocalTestnet => ChainSpec::from_genesis(
                "Local Testnet",
                "local_testnet",
                || {
                    testnet_genesis(
                        vec![authority_key("Alice"), authority_key("Bob")],
                        vec![account_key("Alice"), account_key("Bob")],
                        account_key("Alice"),
                    )
                },
                vec![],
                None,
                None,
                None,
                None,
            ),
            Alternative::Akropolis => akropolis_genesis()?,
            Alternative::AkropolisStaging => {
                let boot_nodes = vec![
                    "/ip4/157.230.35.215/tcp/30333/p2p/QmdRjsEvcGGKDTPAcVnCrRnsqqhbURbzetkkUQYwAmnxaS".to_string(),
                    "/ip4/178.128.225.241/tcp/30333/p2p/QmbriyUytrn9W2AAsnMXN8g4SGQ8cspnmFju4ZJYiYq1Ax".to_string()
                ];
                let telemetry = TelemetryEndpoints::new(vec![
                    ("ws://telemetry.polkadot.io:1024".to_string(), 0),
                    ("ws://167.99.142.212:1024".to_string(), 0),
                ]);
                ChainSpec::from_genesis(
                    "Akropolis",
                    "akropolis",
                    akropolis_staging_genesis,
                    boot_nodes,
                    Some(telemetry),
                    None,
                    None,
                    None,
                )
            }
        })
    }

    pub(crate) fn from(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(Alternative::Development),
            "local" => Some(Alternative::LocalTestnet),
            "" | "akropolis" => Some(Alternative::Akropolis),
            "akropolis_staging" => Some(Alternative::AkropolisStaging),
            _ => None,
        }
    }
}

fn testnet_genesis(
    initial_authorities: Vec<AuthorityId>,
    endowed_accounts: Vec<AccountId>,
    root_key: AccountId,
) -> GenesisConfig {
	let bridge_validators = vec![
        hex!("3a495ac93eca02fa4f64bcc99b2f950b7df8d866b4b107596a0ea7a547b48753").unchecked_into(), // 5DP8Rd8jUQD9oukZduPSMxdrH8g3r4mzS1zXLZCS6qDissTm
        hex!("1450cad95384831a1b267f2d18273b83b77aaee8555a23b7f1abbb48b5af8e77").unchecked_into(), // 5CXLpEbkeqp475Y8p7uMeiimgKXX6haZ1fCT4jzyry26CPxp
        hex!("2452305cbdb33a55de1bc46f6897fd96d724d8bccc5ca4783f6f654af8582d58").unchecked_into(), // 5CtKzjXcWrD8GRQqorFiwHF9oUbx2wHpf43erxB2u7dpfCq9
    ];
    GenesisConfig {
		consensus: Some(ConsensusConfig {
			code: include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/erc20_substrate_bridge_runtime_wasm.compact.wasm").to_vec(),
			authorities: initial_authorities.clone(),
		}),
		system: None,
		timestamp: Some(TimestampConfig {
			minimum_period: 5, // 10 second block time.
		}),
		indices: Some(IndicesConfig {
			ids: endowed_accounts.clone(),
		}),
		balances: Some(BalancesConfig {
			transaction_base_fee: 1,
			transaction_byte_fee: 0,
			existential_deposit: 500,
			transfer_fee: 0,
			creation_fee: 0,
			balances: endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
			vesting: vec![],
		}),
		sudo: Some(SudoConfig {
			key: root_key,
		}),
		session: Some(SessionConfig {
			validators: endowed_accounts.clone(),
			keys: endowed_accounts.iter().cloned().zip(initial_authorities.clone()).collect(),
			session_length: 6
		}),
		staking: Some(StakingConfig {
			validator_count: 5, // The ideal number of staking participants.
			minimum_validator_count: 1, // Minimum number of staking participants before emergency conditions are imposed
			sessions_per_era: 5, // The length of a staking era in sessions.
			session_reward: Perbill::from_millionths(10_000), // Maximum reward, per validator, that is provided per acceptable session.
			offline_slash: Perbill::from_percent(50_000), // Slash, per validator that is taken for the first time they are found to be offline.
			offline_slash_grace: 3, // Number of instances of offline reports before slashing begins for validators.
			bonding_duration: 30, // The length of the bonding duration in blocks.
			invulnerables: vec![], // Any validators that may never be slashed or forcibly kicked.
			stakers: vec![], // This is keyed by the stash account.
			current_era: 0, // The current era index.
			current_session_reward: 10, // Maximum reward, per validator, that is provided per acceptable session.
		}),
		democracy: Some(DemocracyConfig {
			launch_period: 1440, // How often (in blocks) new public referenda are launched.
			minimum_deposit: 10_000, // The minimum amount to be used as a deposit for a public referendum proposal.
			public_delay: 5, // The delay before enactment for all public referenda.
			max_lock_periods: 60, // The maximum number of additional lock periods a voter may offer to strengthen their vote.
			voting_period: 144, // How often (in blocks) to check for new votes.
		}),
		council_voting: Some(CouncilVotingConfig {
			cooloff_period: 360, // Period (in blocks) that a veto is in effect.
			voting_period: 60, // Period (in blocks) that a vote is open for.
			enact_delay_period: 5, // Number of blocks by which to delay enactment of successful.
		}),
		grandpa: Some(GrandpaConfig {
			authorities: initial_authorities.iter().cloned().map(|x| (x, 1)).collect()
		}),
		treasury: Some(TreasuryConfig {
			proposal_bond: Permill::from_millionths(50_000), // Proportion of funds that should be bonded in order to place a proposal.
			proposal_bond_minimum: 1_000_000, // Minimum amount of funds that should be placed in a deposit for making a proposal.
			spend_period: 360, // Period between successive spends.
			burn: Permill::from_millionths(100_000), // Percentage of spare funds (if any) that are burnt per spend period.
		}),
		contract: Some(ContractConfig {
			transfer_fee: 100, // The fee required to make a transfer.
			creation_fee: 100, // The fee required to create an account.
			transaction_base_fee: 21, // The fee to be paid for making a transaction; the base.
			transaction_byte_fee: 1, // The fee to be paid for making a transaction; the per-byte portion.
			contract_fee: 21, // The fee required to create a contract instance.
			call_base_fee: 135, // The base fee charged for calling into a contract.
			create_base_fee: 175, // The base fee charged for creating a contract.
			gas_price: 1, // The price of one unit of gas.
			max_depth: 100, // The maximum nesting level of a call/create stack.
			block_gas_limit: 10_000_000, // The maximum amount of gas that could be expended per block.
			current_schedule: Schedule::default(), // Current cost schedule for contracts.
		}),
		bridge: Some(BridgeConfig {
			validator_accounts: bridge_validators,
			validators_count: 3u32,
			pending_burn_limit: 1000,
			pending_mint_limit: 1000,
		}),
	}
}

fn akropolis_genesis() -> Result<ChainSpec, String> {
    ChainSpec::from_embedded(include_bytes!("../res/akropolis.json"))
}

fn akropolis_staging_genesis() -> GenesisConfig {
    let endowed_accounts = vec![
        hex!("a44d98789c9a618560cdfba9b9100df8f74cf8a477e71f202a841a5bd3b7d040").unchecked_into(), // 5Fn8m67WboHonj6SjHogaUkQnTEyLwkAkimkyDvC5mFriuea
    ];

    let bridge_validators = vec![
        hex!("3a495ac93eca02fa4f64bcc99b2f950b7df8d866b4b107596a0ea7a547b48753").unchecked_into(), // 5DP8Rd8jUQD9oukZduPSMxdrH8g3r4mzS1zXLZCS6qDissTm
        hex!("1450cad95384831a1b267f2d18273b83b77aaee8555a23b7f1abbb48b5af8e77").unchecked_into(), // 5CXLpEbkeqp475Y8p7uMeiimgKXX6haZ1fCT4jzyry26CPxp
        hex!("2452305cbdb33a55de1bc46f6897fd96d724d8bccc5ca4783f6f654af8582d58").unchecked_into(), // 5CtKzjXcWrD8GRQqorFiwHF9oUbx2wHpf43erxB2u7dpfCq9
    ];

    let initial_authorities = vec![
        // (stash, controller, session)
        (
            hex!("2c5d2a346ed26c77f8f751c231e88115a9894557bf9ef188b051c2be6ae8593e")
                .unchecked_into(), // 5D4se4ee1pnN4vzuRQf4i8w6mLcYACgnxvwCPqJU7wTxCFDF
            hex!("060d81df438bd61b7c58aa180c376199a929065cda5b8999bdad7600d61ee23f")
                .unchecked_into(), // 5CCeA4L5puxt5UiM9g6f5mrLHc6LHA4vWCaAxsU6p5mvXoUX
            hex!("c09c516fe5f616b7a8a17bbd52372ba66615a969a83e27321591ca0a335faf1b")
                .unchecked_into(), // 5GRFVDCnrXbqJZ1xpEKiu3KigLVsfQiudNieGfKafK1q63XJ
        ),
        (
            hex!("d84358df95f03cfa0392a99d9d4f774e28e36f53922ef574af7a54a4201ae60a")
                .unchecked_into(), // 5GxGCw4ZtrFyKJwEXSe2P6pzL4mmbQAfWzjnqNUGJWGANVU6
            hex!("5a70788f479ef92863cdfba2aa63f4e18fd2609fe6c922d33b8d5416a173af22")
                .unchecked_into(), // 5E7HaTQL2f9V6RoyRnZcJq1YdscnT6GQWM92KDbUNQtbMq8K
            hex!("648d7d9895dc548daf42496d791f7b636047548538cbe3aaf64538bf18335006")
                .unchecked_into(), // 5ELYgE69fvCDsKRaH2zELPHNqMGHDPdaTx7XfeVM9iTLmc3F
        ),
        (
            hex!("de97817f71aa70f005a692e49a7bf59ce1e4591f5f6a174c3a72f28440e23f01")
                .unchecked_into(), // 5H6ZVckfsC2MmCEMiNSgANb59hG1A1YZ4R4htpSVQRaa5g9C
            hex!("eafbee6e929feb39c392798c1eb8ffad9db77b402b4d9c41329ca6eb7295883f")
                .unchecked_into(), // 5HNouEfpMzyBDpbnb5y3TmpCVUBiEBZY5ohmG2G2LdTNFVE2
            hex!("506df52cd7366c5755f0dea6cc77e214dd1c5f89d6df03d300792931898d4074")
                .unchecked_into(), // 5DtAMP4KBredD7Jb7wxdqBwPpmyppBeXhprNsxKmbhX16Cw8
        ),
    ];

    const DEV: u128 = 1_000_000_000_000_000;
    const ENDOWMENT: u128 = 4_000_000 * DEV;
    const STASH: u128 = 10 * DEV;

    let balances = endowed_accounts
        .iter()
        .cloned()
        .map(|x| (x, ENDOWMENT))
        .chain(initial_authorities.iter().cloned().map(|x| (x.0, STASH)))
        .chain(bridge_validators.iter().cloned().map(|x| (x, STASH)))
        .collect();

    GenesisConfig {
		consensus: Some(ConsensusConfig {
			code: include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/erc20_substrate_bridge_runtime_wasm.compact.wasm").to_vec(),
			authorities: initial_authorities.iter().cloned().map(|x| x.2).collect(),
		}),
		system: None,
		timestamp: Some(TimestampConfig {
			minimum_period: 5, // 10 second block time.
		}),
		indices: Some(IndicesConfig {
			ids: endowed_accounts.iter().cloned().chain(initial_authorities.iter().cloned().map(|x| x.0)).collect(),
		}),
		balances: Some(BalancesConfig {
			transaction_base_fee: 1,
			transaction_byte_fee: 0,
			existential_deposit: 500,
			transfer_fee: 0,
			creation_fee: 0,
			balances,
			vesting: vec![],
		}),
		sudo: Some(SudoConfig {
			key: endowed_accounts[0].clone(),
		}),
		session: Some(SessionConfig {
			validators: initial_authorities.iter().cloned().map(|x| x.1).collect(),
			keys: initial_authorities.iter().cloned().map(|x| (x.1, x.2)).collect(),
			session_length: 6
		}),
		staking: Some(StakingConfig {
			validator_count: 5, // The ideal number of staking participants.
			minimum_validator_count: 1, // Minimum number of staking participants before emergency conditions are imposed
			sessions_per_era: 5, // The length of a staking era in sessions.
			session_reward: Perbill::from_millionths(10_000), // Maximum reward, per validator, that is provided per acceptable session.
			offline_slash: Perbill::from_percent(50_000), // Slash, per validator that is taken for the first time they are found to be offline.
			offline_slash_grace: 3, // Number of instances of offline reports before slashing begins for validators.
			bonding_duration: 30, // The length of the bonding duration in blocks.
			invulnerables: initial_authorities.iter().cloned().map(|x| x.1).collect(), // Any validators that may never be slashed or forcibly kicked.
			stakers: initial_authorities.iter().cloned().map(|x| (x.0, x.1, STASH, StakerStatus::Validator)).collect(), // This is keyed by the stash account.
			current_era: 0, // The current era index.
			current_session_reward: 10, // Maximum reward, per validator, that is provided per acceptable session.
		}),
		democracy: Some(DemocracyConfig {
			launch_period: 1440, // How often (in blocks) new public referenda are launched.
			minimum_deposit: 10_000, // The minimum amount to be used as a deposit for a public referendum proposal.
			public_delay: 5, // The delay before enactment for all public referenda.
			max_lock_periods: 60, // The maximum number of additional lock periods a voter may offer to strengthen their vote.
			voting_period: 144, // How often (in blocks) to check for new votes.
		}),
		council_voting: Some(CouncilVotingConfig {
			cooloff_period: 360, // Period (in blocks) that a veto is in effect.
			voting_period: 60, // Period (in blocks) that a vote is open for.
			enact_delay_period: 5, // Number of blocks by which to delay enactment of successful.
		}),
		grandpa: Some(GrandpaConfig {
			authorities: initial_authorities.iter().cloned().map(|x| (x.2, 1)).collect()
		}),
		treasury: Some(TreasuryConfig {
			proposal_bond: Permill::from_millionths(50_000), // Proportion of funds that should be bonded in order to place a proposal.
			proposal_bond_minimum: 1_000_000, // Minimum amount of funds that should be placed in a deposit for making a proposal.
			spend_period: 360, // Period between successive spends.
			burn: Permill::from_millionths(100_000), // Percentage of spare funds (if any) that are burnt per spend period.
		}),
		contract: Some(ContractConfig {
			transfer_fee: 100, // The fee required to make a transfer.
			creation_fee: 100, // The fee required to create an account.
			transaction_base_fee: 21, // The fee to be paid for making a transaction; the base.
			transaction_byte_fee: 1, // The fee to be paid for making a transaction; the per-byte portion.
			contract_fee: 21, // The fee required to create a contract instance.
			call_base_fee: 135, // The base fee charged for calling into a contract.
			create_base_fee: 175, // The base fee charged for creating a contract.
			gas_price: 1, // The price of one unit of gas.
			max_depth: 100, // The maximum nesting level of a call/create stack.
			block_gas_limit: 10_000_000, // The maximum amount of gas that could be expended per block.
			current_schedule: Schedule::default(), // Current cost schedule for contracts.
		}),
		bridge: Some(BridgeConfig {
			validator_accounts: bridge_validators,
			validators_count: 3u32, 
			pending_burn_limit: 1000,
			pending_mint_limit: 1000,
		})
	}
}

use erc20_substrate_bridge_runtime::{
    AccountId, BalancesConfig, ConsensusConfig, ContractConfig, CouncilVotingConfig,
    DemocracyConfig, GenesisConfig, GrandpaConfig, IndicesConfig, Perbill, Permill, Schedule,
    SessionConfig, StakerStatus, StakingConfig, SudoConfig, TimestampConfig, TreasuryConfig,
	BridgeConfig
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
			validators_count: 3usize, //default 
			_genesis_phantom_data: Default::default(), //https://substrate.dev/docs/en/runtime/initializing-storage#calculate-individually-with-build
		}),
	}
}

fn akropolis_genesis() -> Result<ChainSpec, String> {
    ChainSpec::from_embedded(include_bytes!("../res/akropolis.json"))
}

fn akropolis_staging_genesis() -> GenesisConfig {
    let endowed_accounts = vec![
        hex!("ac093ae2c4b5cc62aca5ceca961ed3bd3ad65d0fdcc3cbd206109d5ab970e171").unchecked_into(), // 5FxGqPvuyvKaGvwaHAiTjvVpQMoZcgd1tLbWWWyPH4QNyc6Q
    ];

    let initial_authorities = vec![
        // (stash, controller, session)
        (
            hex!("927b39cac18dabc394d7c744fad4467d51310bf299330f9810427f8508d6ee09")
                .unchecked_into(), // 5FNmTHadRw12fPwkrSdoKNznX5HpTcvcwvmKu5PF1suGiwP8
            hex!("6cd3b2029a60d1e8a415de9aeed40b76ed3815678f75557b12db2b57559f8d43")
                .unchecked_into(), // 5EXPv3Y6obajCD9PTCa4u6ZdZgQ2wowFh8yZA7DiKibirpDW
            hex!("c763486fcc0753cfde644da6d193d092d10015384cb5ef6cca7597bbb9a900b3")
                .unchecked_into(), // 5Ga8sxc52JGnb31zhizJpS9ixVzMneDjse8XLNAi4Gvp2mhB
        ),
        (
            hex!("10fffd9162e7950a449eff6024ac326321228df2659c2a1f9d5c084c56fcc112")
                .unchecked_into(), // 5CSzfigG2EGM3MmCcjKSAJMdtgbh4eNKc54kVU9BJBPVxju3
            hex!("4e18e46d6e8c086a81a9162fa72d95bb3a0712f0ab73ea872cc88b810bdd2575")
                .unchecked_into(), // 5Dq6zBbF78utVLB16oAc3b7bCJKRksuoomoWeBF7LsbKVcjx
            hex!("a17221f222c706dea7adfb7e6ec3dbba9a7febc8eed6ff3aa5428db31a16c875")
                .unchecked_into(), // 5FiPUGuYULQhcxkdUhAakHprBFQWj37ac5YwaSo5Kph9Vypz
        ),
    ];

    const DEV: u128 = 1_000_000_000_000_000;
    const ENDOWMENT: u128 = 4_000_000 * DEV;
    const STASH: u128 = 10 * DEV;

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
			balances: endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).chain(initial_authorities.iter().cloned().map(|x| (x.0, STASH))).collect(),
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
			validators_count: 3usize, //default 
			_genesis_phantom_data: Default::default(), //https://substrate.dev/docs/en/runtime/initializing-storage#calculate-individually-with-build
		})
	}
}

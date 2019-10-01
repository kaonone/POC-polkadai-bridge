# erc20-substrate-bridge

A new SRML-based Substrate node, ready for hacking.

# Building

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Install required tools:

```bash
./scripts/init.sh
```

Build the WebAssembly binary:

```bash
./scripts/build.sh
```

Build all native code:

```bash
cargo build
```

# Run

You can start a full node:

```bash
cargo run -- --name node-name
```

# Development

You can start a development chain with:

```bash
cargo run -- --dev
```

Detailed logs may be shown by running the node with the following environment variables set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

If you want to see the multi-node consensus algorithm in action locally, then you can create a local testnet with two validator nodes for Alice and Bob, who are the initial authorities of the genesis chain that have been endowed with testnet units. Give each node a name and expose them so they are listed on the Polkadot [telemetry site](https://telemetry.polkadot.io/#/Local%20Testnet). You'll need two terminal windows open.

We'll start Alice's substrate node first on default TCP port 30333 with her chain database stored locally at `/tmp/alice`. The bootnode ID of her node is `QmQZ8TjTqeDj3ciwr93EJ95hxfDsb9pEYDizUAbWpigtQN`, which is generated from the `--node-key` value that we specify below:

```bash
cargo run -- \
  --base-path /tmp/alice \
  --chain=local \
  --alice \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --validator
```

In the second terminal, we'll start Bob's substrate node on a different TCP port of 30334, and with his chain database stored locally at `/tmp/bob`. We'll specify a value for the `--bootnodes` option that will connect his node to Alice's bootnode ID on TCP port 30333:

```bash
cargo run -- \
  --base-path /tmp/bob \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmQZ8TjTqeDj3ciwr93EJ95hxfDsb9pEYDizUAbWpigtQN \
  --chain=local \
  --bob \
  --port 30334 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --validator
```

Additional CLI usage options are available and may be shown by running `cargo run -- --help`.


# How it works

## Account creation


This guide will walk you through how to create account and how to connect to AkropolisOSChain Testnet.

1) Open [Akropolis UI](https://wallet.akropolis.io) (it’s polkadotJS app working with substrate v.1.0). You can also use [Polkadot UI](https://polkadot.js.org/apps/#/explorer).

2) Go to *Settings*, open *Developer* tab. Insert in textbox description of types (copy&paste from here) and Save it.


```bash

{
  "Count": "u64",
  "DaoId": "u64",
  "MemberId": "u64",
  "ProposalId": "u64",
  "TokenBalance": "u64",
  "VotesCount": "MemberId",
  "TokenId": "u32",
  "Days": "u32",
  "Rate": "u32",
  "Dao": {
    "address": "AccountId",
    "name": "Text",
    "description": "Bytes",
    "founder": "AccountId"
  },
  "Action": {
    "_enum": {
      "EmptyAction": null,
      "AddMember": "AccountId",
      "RemoveMember": "AccountId",
      "GetLoan": "(Vec<u8>, Days, Rate, Balance)",
      "Withdraw": "(AccountId, Balance, Vec<u8>)"
    }
  },
  "Proposal": {
    "dao_id": "DaoId",
    "action": "Action",
    "open": "bool",
    "accepted": "bool",
    "voting_deadline": "BlockNumber",
    "yes_count": "VotesCount",
    "no_count": "VotesCount"
  },
  "Token": {
    "token_id": "u32",
    "decimals": "u16",
    "symbol": "Vec<u8>"
  },
  "Status": {
      "_enum":[
        "Pending",
        "Deposit",
        "Withdraw",
        "Approved",
        "Canceled",
        "Confirmed"
      ]
  },
    "Message": {
      "message_id": "H256",
      "eth_address": "H160",
      "substrate_address": "AccountId",
      "amount": "TokenBalance",
      "status": "Status",
      "direction": "Status"
  },
  "BridgeTransfer": {
    "transfer_id": "ProposalId",
    "message_id": "H256",
    "open": "bool",
    "votes": "MemberId"
  }
}



```

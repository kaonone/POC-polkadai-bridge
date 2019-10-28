[![version](https://img.shields.io/badge/version-0.2-blue)](https://medium.com/akropolis)

# DAI Ethereum  <-> Parity Substrate Bridge
 Ethereum <-> Parity Substrate Blockchain bridge for self transfers of DAI Token (ERC20) to sDAI (ERC20 representation).

## You can try it out in our chain:
1. Make sure you have Ethereum and Substrate extensions. Typical choices would be:
  <br>a. `Metamask` (or any other Ethereum extension) and switch it to `Kovan`
  <br>b. `polkadot{.js}`
2. Go [here](https://polkadai-bridge.akropolis.io/)
3. Connect with both extensions(two pop-up windows should appear)
4. You will see that your balances from extensions should appear on the page.
5. Transfer some Kovan test DAI to our Substrate-based chain.
6. Transfer some DAI from our chain to your Ethereum account.

It should be pretty obvious from this point.
If you hit any problems, please feel free to file an issue!

<pre>
├── bridge
│   ├── ethereum
│   ├── frontend
│   └── validator - Validator service to connect Substrate to Ethereum.
├── runtime
├── scripts
├── src
</pre>

## Building

### 1. Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

### 2. Install required tools:

```bash
./scripts/init.sh
```

### 3. Build the WebAssembly binary:

```bash
./scripts/build.sh
```

### 4. Build all native code:

```bash
cargo build
```

### 5. Build the validator node

Follow the [validator README](https://github.com/akropolisio/erc20-substrate-bridge/blob/master/bridge/validator/README.md) instructions.

### 6. Build frontend

Follow the [frontend README](https://github.com/akropolisio/erc20-substrate-bridge/blob/master/bridge/frontend/README.md) instructions.

### 4.(Optional) Tweak configuration to use your keys and account.

## Run

### Start a full node:

```bash
cargo run -- --name node-name
```



### Development

You can start a development chain with:

```bash
cargo run -- --dev
```

Detailed logs may be shown by running the node with the following environment variables set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

If you want to see the multi-node consensus algorithm in action locally, then you can create a local testnet with two validator nodes for Alice and Bob, who are the initial authorities of the genesis chain that have been endowed with testnet units. Give each node a name and expose them so they are listed on the Polkadot [telemetry site](https://telemetry.polkadot.io/#/Local%20Testnet). You'll need two terminal windows open.

We'll start Alice's Substrate node first on default TCP port 30333 with her chain database stored locally at `/tmp/alice`. The bootnode ID of her node is `QmQZ8TjTqeDj3ciwr93EJ95hxfDsb9pEYDizUAbWpigtQN`, which is generated from the `--node-key` value that we specify below:

```bash
cargo run -- \
  --base-path /tmp/alice \
  --chain=local \
  --alice \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --validator
```

In the second terminal, we'll start Bob's Substrate node on a different TCP port of 30334, and with his chain database stored locally at `/tmp/bob`. We'll specify a value for the `--bootnodes` option that will connect his node to Alice's bootnode ID on TCP port 30333:

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


## How it works

## Make bridge on your chain
In case you want to test it in a customise-all-the-things fashion, buckle up for some extra work!

This guide will walk you through how to create an account and how to connect to AkropolisOSChain Testnet.
1) Run the node (previous steps, Build -> Run)

2) Open [Akropolis UI](https://wallet.akropolis.io) (it’s polkadotJS app working with Substrate v.1.0). You can also use [Polkadot UI](https://polkadot.js.org/apps/#/explorer).

3) Go to *Settings*, open *Developer* tab. Insert in textbox description of types (copy&paste from here) and Save it.


```bash

{
  "MemberId": "u64",
  "ProposalId": "u64",
  "TokenBalance": "u128",
  "TokenId": "u32",
  "Token": {
    "token_id": "u32",
    "decimals": "u16",
    "symbol": "Vec<u8>"
  },
  "Status": {
      "_enum":[
        "Revoked",
        "Pending",
        "PauseTheBridge",
        "ResumeTheBridge",
        "AddValidator",
        "RemoveValidator",
        "ChangeMinTx",
        "ChangeMaxTx",
        "Deposit",
        "Withdraw",
        "Approved",
        "Canceled",
        "Confirmed"
      ]
  },
  "Kind" :{
    "_enum":[
    "Transfer",
    "Limits",
    "Validator",
    "Bridge",
  },
    "TransferMessage": {
      "message_id": "H256",
      "eth_address": "H160",
      "substrate_address": "AccountId",
      "amount": "TokenBalance",
      "status": "Status",
      "direction": "Status"
  },
    "LimitMessage": {
      "message_id": "H256",
      "amount": "TokenBalance",
      "status": "Status",
      "action": "Status"
  },
    "BridgeMessage": {
      "message_id": "H256",
      "account": "AccountId",
      "status": "Status",
      "action": "Status"
  },
  "BridgeTransfer": {
    "transfer_id": "ProposalId",
    "message_id": "H256",
    "open": "bool",
    "votes": "MemberId"
    "kind": "Kind"
  },
}

```

4) Create an account for each validator you want to launch. 
Go to *Accounts* and generate new account(and modify validators mnemonic phrase in [ .env file](https://github.com/akropolisio/erc20-substrate-bridge/blob/master/bridge/validator/.env.example)) for each validator.

5) Repeat step 4 for each validator in case you have more than one.

6) Modify validators in [chain_spec.rs](https://github.com/akropolisio/erc20-substrate-bridge/blob/master/src/chain_spec.rs) in GenesisConfig -> bridge

7) Repeat Build + Run instructions 

8) Launch bridge/frontend(you also might need to tweak the keys and endpoints there)

9) Enjoy your local ERC20 Substrate <--> Ethereum bridge in [your browser](http://localhost:1234/)



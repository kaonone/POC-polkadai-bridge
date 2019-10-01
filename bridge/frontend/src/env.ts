import { RegistryTypes } from '@polkadot/types/types';

interface INetworkConfig {
  id: number;
  name: string;
  rpcUrl: string;
  contracts: {
    bridge: string;
    dai: string;
  };
  etherskanDomain: string;
}

const ethNetworkConfigs: Record<number, INetworkConfig> = {
  "42": {
    id: 42,
    name: "Kovan",
    rpcUrl: "https://kovan.infura.io/",
    contracts: {
      bridge: "0x9ff8c644F09B0B7dc030C8aaD52dC1628a22C4c2",
      dai: "0xC4375B7De8af5a38a93548eb8453a498222C4fF2"
    },
    etherskanDomain: 'https://kovan.etherscan.io/',
  },
  "1": {
    id: 1,
    name: "Mainnet",
    rpcUrl: "https://mainnet.infura.io/",
    contracts: {
      bridge: "0x9ff8c644F09B0B7dc030C8aaD52dC1628a22C4c2",
      dai: "0x89d24a6b4ccb1b6faa2625fe562bdd9a23260359"
    },
    etherskanDomain: 'https://etherscan.io/',
  }
};

export const NETWORK_ID = 42;
export const ETH_NETWORK_CONFIG = ethNetworkConfigs[NETWORK_ID];
export const DEFAULT_DECIMALS = 18;

export const SUBSTRATE_DEFAULT_ADDRESS_PREFIX = 42;
export const SUBSTRATE_NODE_URL = 'wss://node1-chain.akropolis.io';
export const SUBSTRATE_NODE_CUSTOM_TYPES: RegistryTypes = {
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
  } as any,
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
    "symbol": "Vec<u8>"
  },
  "Status": {
    "_enum": [
      "Pending",
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
    "status": "Status"
  },
  "BridgeTransfer": {
    "transfer_id": "ProposalId",
    "message_id": "H256",
    "open": "bool",
    "votes": "MemberId"
  }
};

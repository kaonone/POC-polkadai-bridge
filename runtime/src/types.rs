use parity_codec::{Decode, Encode};
use primitives::H160;
use rstd::prelude::Vec;

pub type MemberId = u64;
pub type ProposalId = u64;

// token factory types
pub type TokenBalance = u128;
pub type TokenId = u32;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Token {
    pub id: TokenId,
    pub decimals: u16,
    pub symbol: Vec<u8>,
}

// bridge types
#[derive(Encode, Decode, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BridgeTransfer<Hash> {
    pub transfer_id: ProposalId,
    pub message_id: Hash,
    pub open: bool,
    pub votes: MemberId,
    pub kind: Kind,
}

#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Status {
    Revoked,
    Pending,
    PauseTheBridge,
    ResumeTheBridge,
    AddValidator,
    RemoveValidator,
    ChangeMinTx,
    ChangeMaxTx,
    ChangePendingBurnLimit,
    ChangePendingMintLimit,
    Deposit,
    Withdraw,
    Approved,
    Canceled,
    Confirmed,
}

#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Kind {
    Transfer,
    Limits,
    Validator,
    Bridge,
}

#[derive(Encode, Decode, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct TransferMessage<AccountId, Hash> {
    pub message_id: Hash,
    pub eth_address: H160,
    pub substrate_address: AccountId,
    pub amount: TokenBalance,
    pub status: Status,
    pub action: Status,
}

#[derive(Encode, Decode, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct LimitMessage<Hash> {
    pub message_id: Hash,
    pub amount: TokenBalance,
    pub action: Status,
    pub status: Status,
}

#[derive(Encode, Decode, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BridgeMessage<AccountId, Hash> {
    pub message_id: Hash,
    pub account: AccountId,
    pub action: Status,
    pub status: Status,
}

impl<A, H> Default for TransferMessage<A, H>
where
    A: Default,
    H: Default,
{
    fn default() -> Self {
        TransferMessage {
            message_id: H::default(),
            eth_address: H160::default(),
            substrate_address: A::default(),
            amount: TokenBalance::default(),
            status: Status::Withdraw,
            action: Status::Withdraw,
        }
    }
}

impl<H> Default for LimitMessage<H>
where
    H: Default,
{
    fn default() -> Self {
        LimitMessage {
            message_id: H::default(),
            amount: TokenBalance::default(),
            status: Status::ChangeMinTx,
            action: Status::ChangeMinTx,
        }
    }
}

impl<A, H> Default for BridgeMessage<A, H>
where
    A: Default,
    H: Default,
{
    fn default() -> Self {
        BridgeMessage {
            message_id: H::default(),
            account: A::default(),
            action: Status::Revoked,
            status: Status::Revoked,
        }
    }
}

impl<H> Default for BridgeTransfer<H>
where
    H: Default,
{
    fn default() -> Self {
        BridgeTransfer {
            transfer_id: ProposalId::default(),
            message_id: H::default(),
            open: true,
            votes: MemberId::default(),
            kind: Kind::Transfer,
        }
    }
}
/// runtime module implementing Substrate side of AkropolisOS token exchange bridge
/// You can use mint to create tokens backed by locked funds on Ethereum side
/// and transfer tokens on substrate side freely
///
use crate::token;
use crate::types::{MemberId, ProposalId, TokenBalance};
use parity_codec::{Decode, Encode};
use primitives::H160;
use runtime_primitives::traits::{As, Hash};
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, StorageMap, StorageValue,
};
use system::{self, ensure_signed};

#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BridgeTransfer<Hash> {
    transfer_id: ProposalId,
    message_id: Hash,
    open: bool,
    votes: MemberId,
}

#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
enum Status {
    Pending,
    Deposit,
    Withdraw,
    Approved,
    Canceled,
    Confirmed,
}

#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Message<AccountId, Hash> {
    message_id: Hash,
    eth_address: H160,
    substrate_address: AccountId,
    amount: TokenBalance,
    status: Status,
    direction: Status,
}

impl<A, H> Default for Message<A, H>
where
    A: Default,
    H: Default,
{
    fn default() -> Self {
        Message {
            message_id: H::default(),
            eth_address: H160::default(),
            substrate_address: A::default(),
            amount: TokenBalance::default(),
            status: Status::Withdraw,
            direction: Status::Withdraw,
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
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Hash = <T as system::Trait>::Hash,
    {
        RelayMessage(Hash),
        ApprovedRelayMessage(Hash, AccountId, H160, TokenBalance),
        Minted(Hash),
        Burned(Hash, AccountId, H160, TokenBalance),
    }
);

pub trait Trait: token::Trait + system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Bridge {
        BridgeTransfers get(transfers): map ProposalId => BridgeTransfer<T::Hash>;
        BridgeTransfersCount get(bridge_transfers_count): ProposalId;
        Messages get(messages): map(T::Hash) => Message<T::AccountId, T::Hash>;
        TransferId get(transfer_id_by_hash): map(T::Hash) => ProposalId;
        MessageId get(message_id_by_transfer_id): map(ProposalId) => T::Hash;

        ValidatorsCount get(validators_count) config(): usize = 3;
        ValidatorsAccounts get(validators_accounts): map MemberId => T::AccountId;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        // initiate substrate -> ethereum transfer.
        // create proposition and emit the RelayMessage event
        fn set_transfer(origin, to: H160, #[compact] amount: TokenBalance)-> Result
        {
            let from = ensure_signed(origin)?;

            let transfer_hash = (&from, &to, amount, T::BlockNumber::sa(0)).using_encoded(<T as system::Trait>::Hashing::hash);

            let message = Message{
                message_id: transfer_hash,
                eth_address: to,
                substrate_address: from,
                amount,
                status: Status::Withdraw,
                direction: Status::Withdraw,
            };
            Self::get_transfer_id_checked(transfer_hash)?;
            Self::deposit_event(RawEvent::RelayMessage(transfer_hash));

            <Messages<T>>::insert(transfer_hash, message);
            Ok(())
        }

        // ethereum-side multi-signed mint operation
        fn multi_signed_mint(origin, message_id: T::Hash, from: H160, to: T::AccountId, #[compact] amount: TokenBalance)-> Result {
            ensure_signed(origin)?;

            if !<Messages<T>>::exists(message_id) {
                let message = Message{
                    message_id,
                    eth_address: from,
                    substrate_address: to,
                    amount,
                    status: Status::Deposit,
                    direction: Status::Deposit,
                };
                <Messages<T>>::insert(message_id, message);
                Self::get_transfer_id_checked(message_id)?;
            }

            let transfer_id = <TransferId<T>>::get(message_id);
            Self::_sign(transfer_id)?;

            Ok(())
        }

        // validator`s response to RelayMessage
        fn approve_transfer(origin, message_id: T::Hash) -> Result {
            ensure_signed(origin)?;
            let id = <TransferId<T>>::get(message_id);

            Self::_sign(id)
        }

        //confirm burn from validator
        fn confirm_transfer(origin, message_id: T::Hash) -> Result {
            ensure_signed(origin)?;
            let id = <TransferId<T>>::get(message_id);

            let is_approved = <Messages<T>>::get(message_id).status == Status::Approved ||
            <Messages<T>>::get(message_id).status == Status::Confirmed;
            ensure!(is_approved, "This transfer must be approved first.");

            Self::update_status(message_id, Status::Confirmed)?;
            Self::reopen_for_burn_confirmation(message_id)?;
            Self::_sign(id)?;

            Ok(())
        }

        //cancel burn from validator
        fn cancel_transfer(origin, message_id: T::Hash) -> Result {
            ensure_signed(origin)?;
            let mut message = <Messages<T>>::get(message_id);
            message.status = Status::Canceled;

            <token::Module<T>>::unlock(&message.substrate_address, message.amount)?;
            <Messages<T>>::insert(message_id, message);

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn _sign(transfer_id: ProposalId) -> Result {
        let mut transfer = <BridgeTransfers<T>>::get(transfer_id);
        let mut message = <Messages<T>>::get(transfer.message_id);
        ensure!(transfer.open, "This transfer is not open");

        transfer.votes += 1;

        if Self::votes_are_enough(transfer.votes) {
            match message.status {
                Status::Confirmed => (), // if burn is confirmed
                _ => message.status = Status::Approved,
            };
            Self::execute_transfer(message)?;
            transfer.open = false;
        } else {
            match message.status {
                Status::Confirmed => (),
                _ => Self::update_status(transfer.message_id, Status::Pending)?,
            };
        }

        <BridgeTransfers<T>>::insert(transfer_id, transfer);

        Ok(())
    }

    ///ensure that such transfer exist
    fn get_transfer_id_checked(transfer_hash: T::Hash) -> Result {
        if !<TransferId<T>>::exists(transfer_hash) {
            Self::create_transfer(transfer_hash)?;
        }

        Ok(())
    }

    fn votes_are_enough(votes: MemberId) -> bool {
        votes as f64 / Self::validators_count() as f64 >= 0.51
    }

    /// lock funds after set_transfer call
    fn lock_for_burn(account: T::AccountId, amount: TokenBalance) -> Result {
        <token::Module<T>>::lock(account, amount)?;

        Ok(())
    }

    fn execute_burn(message_id: T::Hash) -> Result {
        let message = <Messages<T>>::get(message_id);
        let from = message.substrate_address.clone();
        let to = message.eth_address;

        <token::Module<T>>::unlock(&from, message.amount)?;
        <token::Module<T>>::_burn(from.clone(), message.amount)?;

        Self::deposit_event(RawEvent::Burned(message_id, from, to, message.amount));
        Ok(())
    }

    fn execute_transfer(message: Message<T::AccountId, T::Hash>) -> Result {
        match message.direction {
            Status::Deposit => match message.status {
                Status::Approved => {
                    let to = message.substrate_address.clone();
                    <token::Module<T>>::_mint(to, message.amount)?;
                    Self::deposit_event(RawEvent::Minted(message.message_id));
                    Self::update_status(message.message_id, Status::Confirmed)
                }
                _ => Err("tried to deposit with non-supported status"),
            },
            Status::Withdraw => match message.status {
                Status::Confirmed => Self::execute_burn(message.message_id),
                Status::Approved => {
                    let to = message.eth_address.clone();
                    let from = message.substrate_address.clone();
                    Self::lock_for_burn(from.clone(), message.amount)?;
                    Self::deposit_event(RawEvent::ApprovedRelayMessage(
                        message.message_id,
                        from,
                        to,
                        message.amount,
                    ));
                    Self::update_status(message.message_id, Status::Approved)
                }
                _ => Err("tried to withdraw with non-supported status"),
            },
            _ => Err("tried to execute transfer with non-supported status"),
        }
    }
    fn create_transfer(transfer_hash: T::Hash) -> Result {
        ensure!(
            !<TransferId<T>>::exists(transfer_hash),
            "This transfer already open"
        );

        let transfer_id = <BridgeTransfersCount<T>>::get();
        let bridge_transfers_count = <BridgeTransfersCount<T>>::get();
        let new_bridge_transfers_count = bridge_transfers_count
            .checked_add(1)
            .ok_or("Overflow adding a new bridge transfer")?;

        let transfer = BridgeTransfer {
            transfer_id,
            message_id: transfer_hash,
            open: true,
            votes: 0,
        };

        <BridgeTransfers<T>>::insert(transfer_id, transfer);
        <BridgeTransfersCount<T>>::mutate(|count| *count += new_bridge_transfers_count);
        <TransferId<T>>::insert(transfer_hash, transfer_id);
        <MessageId<T>>::insert(transfer_id, transfer_hash);

        Ok(())
    }

    fn update_status(id: T::Hash, status: Status) -> Result {
        let mut message = <Messages<T>>::get(id);
        message.status = status;
        <Messages<T>>::insert(id, message);
        Ok(())
    }
    fn reopen_for_burn_confirmation(message_id: T::Hash) -> Result {
        let message = <Messages<T>>::get(message_id);
        let transfer_id = <TransferId<T>>::get(message_id);
        let mut transfer = <BridgeTransfers<T>>::get(transfer_id);
        if transfer.open == false && message.status == Status::Confirmed {
            transfer.votes = 0;
            transfer.open = true;
            <BridgeTransfers<T>>::insert(transfer_id, transfer);
        }
        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H160, H256};
    use runtime_io::with_externalities;
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use support::{assert_noop, assert_ok, impl_outer_origin};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type Log = DigestItem;
    }
    impl balances::Trait for Test {
        type Balance = u128;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type TransactionPayment = ();
        type TransferPayment = ();
        type DustRemoval = ();
        type Event = ();
    }
    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
    }
    impl token::Trait for Test {
        type Event = ();
    }
    impl Trait for Test {
        type Event = ();
    }

    type BridgeModule = Module<Test>;
    type TokenModule = token::Module<Test>;

    const ETH_MESSAGE_ID: &[u8; 32] = b"0x5617efe391571b5dc8230db92ba65b";
    const ETH_ADDRESS: &[u8; 20] = b"0x00b46c2526ebb8f4c9";
    const V1: u64 = 1;
    const V2: u64 = 2;
    const V3: u64 = 3;
    const USER1: u64 = 4;
    const USER2: u64 = 5;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut r = system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .0;

        r.extend(
            balances::GenesisConfig::<Test> {
                balances: vec![
                    (V1, 100000),
                    (V2, 100000),
                    (V3, 100000),
                    (USER1, 100000),
                    (USER2, 300000),
                ],
                vesting: vec![],
                transaction_base_fee: 0,
                transaction_byte_fee: 0,
                existential_deposit: 500,
                transfer_fee: 0,
                creation_fee: 0,
            }
            .build_storage()
            .unwrap()
            .0,
        );

        r.into()
    }

    #[test]
    fn token_eth2sub_mint_works() {
        with_externalities(&mut new_test_ext(), || {
            let message_id = H256::from(ETH_MESSAGE_ID);
            let eth_address = H160::from(ETH_ADDRESS);

            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                message_id,
                eth_address,
                USER2,
                1000
            ));
            let mut message = BridgeModule::messages(message_id);
            assert_eq!(message.status, Status::Pending);

            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V1),
                message_id,
                eth_address,
                USER2,
                1000
            ));
            message = BridgeModule::messages(message_id);
            assert_eq!(message.status, Status::Confirmed);

            let transfer = BridgeModule::transfers(0);
            assert_eq!(transfer.open, false);

            assert_eq!(TokenModule::balance_of(USER2), 1000);
            assert_eq!(TokenModule::total_supply(), 1000);
        })
    }
    #[test]
    fn token_eth2sub_closed_transfer_fail() {
        with_externalities(&mut new_test_ext(), || {
            let message_id = H256::from(ETH_MESSAGE_ID);
            let eth_address = H160::from(ETH_ADDRESS);

            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                message_id,
                eth_address,
                USER2,
                1000
            ));
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V1),
                message_id,
                eth_address,
                USER2,
                1000
            ));
            assert_noop!(
                BridgeModule::multi_signed_mint(
                    Origin::signed(V3),
                    message_id,
                    eth_address,
                    USER2,
                    1000
                ),
                "This transfer is not open"
            );
            assert_eq!(TokenModule::balance_of(USER2), 1000);
            assert_eq!(TokenModule::total_supply(), 1000);
            let transfer = BridgeModule::transfers(0);
            assert_eq!(transfer.open, false);

            let message = BridgeModule::messages(message_id);
            assert_eq!(message.status, Status::Confirmed);
        })
    }

    #[test]
    fn token_sub2eth_burn_works() {
        with_externalities(&mut new_test_ext(), || {
            let eth_message_id = H256::from(ETH_MESSAGE_ID);
            let eth_address = H160::from(ETH_ADDRESS);

            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                eth_message_id,
                eth_address,
                USER2,
                1000
            ));
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V1),
                eth_message_id,
                eth_address,
                USER2,
                1000
            ));

            //substrate ----> ETH
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                eth_address,
                500
            ));
            //RelayMessage(message_id) event emitted

            let sub_message_id = BridgeModule::message_id_by_transfer_id(1);
            let get_message = || BridgeModule::messages(sub_message_id);

            let mut message = get_message();
            assert_eq!(message.status, Status::Withdraw);

            //approval
            assert_eq!(TokenModule::locked(USER2), 0);
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V1),
                sub_message_id
            ));
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V2),
                sub_message_id
            ));

            message = get_message();
            assert_eq!(message.status, Status::Approved);

            // at this point transfer is in Approved status and are waiting for confirmation
            // from ethereum side to burn. Funds are locked.
            assert_eq!(TokenModule::locked(USER2), 500);
            assert_eq!(TokenModule::balance_of(USER2), 1000);
            // once it happends, validators call confirm_transfer

            assert_ok!(BridgeModule::confirm_transfer(
                Origin::signed(V2),
                sub_message_id
            ));

            message = get_message();
            let transfer = BridgeModule::transfers(1);
            assert_eq!(message.status, Status::Confirmed);
            assert_eq!(transfer.open, true);
            assert_ok!(BridgeModule::confirm_transfer(
                Origin::signed(V1),
                sub_message_id
            ));
            // assert_ok!(BridgeModule::confirm_transfer(Origin::signed(USER1), sub_message_id));
            //Burned(Hash, AccountId, H160, u64) event emitted

            assert_eq!(TokenModule::balance_of(USER2), 500);
            assert_eq!(TokenModule::total_supply(), 500);
        })
    }
    #[test]
    fn token_sub2eth_burn_fail_skip_approval() {
        with_externalities(&mut new_test_ext(), || {
            let eth_message_id = H256::from(ETH_MESSAGE_ID);
            let eth_address = H160::from(ETH_ADDRESS);

            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                eth_message_id,
                eth_address,
                USER2,
                1000
            ));
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V1),
                eth_message_id,
                eth_address,
                USER2,
                1000
            ));
            assert_eq!(TokenModule::balance_of(USER2), 1000);
            assert_eq!(TokenModule::total_supply(), 1000);

            //substrate ----> ETH
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                eth_address,
                500
            ));
            //RelayMessage(message_id) event emitted

            let sub_message_id = BridgeModule::message_id_by_transfer_id(1);
            let message = BridgeModule::messages(sub_message_id);
            assert_eq!(message.status, Status::Withdraw);

            assert_eq!(TokenModule::locked(USER2), 0);
            // lets say validators blacked out and we
            // try to confirm without approval anyway
            assert_noop!(
                BridgeModule::confirm_transfer(Origin::signed(USER2), sub_message_id),
                "This transfer must be approved first."
            );
        })
    }
}

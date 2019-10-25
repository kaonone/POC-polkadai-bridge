/// runtime module implementing Substrate side of Erc20SubstrateBridge token exchange bridge
/// You can use mint to create tokens backed by locked funds on Ethereum side
/// and transfer tokens on substrate side freely
///
use crate::token;
use crate::types::{
    BridgeMessage, BridgeTransfer, Kind, LimitMessage, MemberId, ProposalId, Status, TokenBalance,
    TransferMessage, ValidatorMessage,
};
use rstd::prelude::Vec;
use parity_codec::Encode;
use primitives::H160;
use runtime_primitives::traits::{As, Hash};
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, fail, StorageMap, StorageValue,
};
use system::{self, ensure_signed};

const MAX_VALIDATORS: u32 = 100_000;
const DAY: u64 = 14_400;

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Hash = <T as system::Trait>::Hash,
    {
        RelayMessage(Hash),
        ApprovedRelayMessage(Hash, AccountId, H160, TokenBalance),
        MintedMessage(Hash),
        BurnedMessage(Hash, AccountId, H160, TokenBalance),
    }
);

pub trait Trait: token::Trait + system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Bridge {
        BridgeIsOperational get(bridge_is_operational): bool = true;
        BridgeMessages get(bridge_messages): map (T::Hash) => BridgeMessage<T::AccountId, T::Hash>;

        MaxLimit get(max_tx_limit): TokenBalance = 1000;
        MinLimit get(min_tx_limit): TokenBalance = 10;
        LimitMessages get(limit_messages): map(T::Hash) => LimitMessage<T::Hash>;
        PendingBurnLimit get(pending_burn_limit) config(): u128;
        PendingMintLimit get(pending_mint_limit) config(): u128;
        PendingBurnCount get(pending_burn_count): u128;
        PendingMintCount get(pending_mint_count): u128;

        BridgeTransfers get(transfers): map ProposalId => BridgeTransfer<T::Hash>;
        BridgeTransfersCount get(bridge_transfers_count): ProposalId;
        TransferMessages get(messages): map(T::Hash) => TransferMessage<T::AccountId, T::Hash>;
        TransferId get(transfer_id_by_hash): map(T::Hash) => ProposalId;
        MessageId get(message_id_by_transfer_id): map(ProposalId) => T::Hash;

        DailyHolds get(daily_holds): map(T::AccountId) => (T::BlockNumber, T::Hash);

        ValidatorsCount get(validators_count) config(): u32 = 3;
        ValidatorVotes get(validator_votes): map(ProposalId, T::AccountId) => bool;
        ValidatorHistory get(validator_history): map (T::Hash) => ValidatorMessage<T::AccountId, T::Hash>;
        Validators get(validators) build(|config: &GenesisConfig<T>| {
            config.validator_accounts.clone().into_iter()
            .map(|acc: T::AccountId| (acc, true)).collect::<Vec<_>>()
        }): map (T::AccountId) => bool;
        ValidatorAccounts get(validator_accounts) config(): Vec<T::AccountId>;
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
            ensure!(Self::bridge_is_operational(), "Bridge is not operational");
            let can_burn = <PendingBurnCount<T>>::get() < <PendingBurnLimit<T>>::get();
            ensure!(can_burn, "Too many pending burn transactions.");
            Self::check_amount(amount)?;

            let transfer_hash = (&from, &to, amount, T::BlockNumber::sa(0)).using_encoded(<T as system::Trait>::Hashing::hash);

            let message = TransferMessage{
                message_id: transfer_hash,
                eth_address: to,
                substrate_address: from,
                amount,
                status: Status::Withdraw,
                action: Status::Withdraw,
            };
            Self::get_transfer_id_checked(transfer_hash, Kind::Transfer)?;
            Self::deposit_event(RawEvent::RelayMessage(transfer_hash));

            <TransferMessages<T>>::insert(transfer_hash, message);
            Ok(())
        }

        // ethereum-side multi-signed mint operation
        fn multi_signed_mint(origin, message_id: T::Hash, from: H160, to: T::AccountId, #[compact] amount: TokenBalance)-> Result {
            let validator = ensure_signed(origin)?;
            ensure!(Self::bridge_is_operational(), "Bridge is not operational");
            let can_mint = <PendingMintCount<T>>::get() < <PendingMintLimit<T>>::get();
            ensure!(can_mint, "Too many pending mint transactions.");

            Self::check_validator(validator.clone())?;
            Self::check_amount(amount)?;

            if !<TransferMessages<T>>::exists(message_id) {
                let message = TransferMessage{
                    message_id,
                    eth_address: from,
                    substrate_address: to,
                    amount,
                    status: Status::Deposit,
                    action: Status::Deposit,
                };
                <TransferMessages<T>>::insert(message_id, message);
                Self::get_transfer_id_checked(message_id, Kind::Transfer)?;
            }

            let transfer_id = <TransferId<T>>::get(message_id);
            Self::_sign(validator, transfer_id)
        }
        // change minimum tx limit
        fn change_min_limit(origin, message_id: T::Hash, #[compact] amount: TokenBalance)-> Result {
            let validator = ensure_signed(origin)?;
            ensure!(Self::bridge_is_operational(), "Bridge is not operational");
            Self::check_validator(validator.clone())?;
            Self::check_limit(amount)?;

            if !<LimitMessages<T>>::exists(message_id) {
                let message = LimitMessage{
                    message_id,
                    amount,
                    action: Status::ChangeMinTx,
                    status: Status::ChangeMinTx,
                };
                <LimitMessages<T>>::insert(message_id, message);
                Self::get_transfer_id_checked(message_id, Kind::Limits)?;
            }

            let transfer_id = <TransferId<T>>::get(message_id);
            Self::_sign(validator, transfer_id)
        }

        // change maximum tx limit
        fn change_max_limit(origin, message_id: T::Hash, #[compact] amount: TokenBalance)-> Result {
            let validator = ensure_signed(origin)?;
            ensure!(Self::bridge_is_operational(), "Bridge is not operational");
            Self::check_validator(validator.clone())?;
            Self::check_limit(amount)?;

            if !<LimitMessages<T>>::exists(message_id) {
                let message = LimitMessage{
                    message_id,
                    amount,
                    action: Status::ChangeMaxTx,
                    status: Status::ChangeMaxTx,
                };
                <LimitMessages<T>>::insert(message_id, message);
                Self::get_transfer_id_checked(message_id, Kind::Limits)?;
            }

            let transfer_id = <TransferId<T>>::get(message_id);
            Self::_sign(validator, transfer_id)
        }

        // set maximum pending burn transaction limit
        fn set_pending_burn_limit(origin, message_id: T::Hash, #[compact] amount: TokenBalance)-> Result {
            let validator = ensure_signed(origin)?;
            ensure!(Self::bridge_is_operational(), "Bridge is not operational");
            Self::check_validator(validator.clone())?;

            if !<LimitMessages<T>>::exists(message_id) {
                let message = LimitMessage{
                    message_id,
                    amount,
                    action: Status::ChangePendingBurnLimit,
                    status: Status::ChangePendingBurnLimit,
                };
                <LimitMessages<T>>::insert(message_id, message);
                Self::get_transfer_id_checked(message_id, Kind::Limits)?;
            }

            let transfer_id = <TransferId<T>>::get(message_id);
            Self::_sign(validator, transfer_id)
        }

        // set maximum pending mint transaction limit
        fn set_pending_mint_limit(origin, message_id: T::Hash, #[compact] amount: TokenBalance)-> Result {
            let validator = ensure_signed(origin)?;
            ensure!(Self::bridge_is_operational(), "Bridge is not operational");
            Self::check_validator(validator.clone())?;

            if !<LimitMessages<T>>::exists(message_id) {
                let message = LimitMessage{
                    message_id,
                    amount,
                    action: Status::ChangePendingMintLimit,
                    status: Status::ChangePendingMintLimit,
                };
                <LimitMessages<T>>::insert(message_id, message);
                Self::get_transfer_id_checked(message_id, Kind::Limits)?;
            }

            let transfer_id = <TransferId<T>>::get(message_id);
            Self::_sign(validator, transfer_id)
        }

        // validator`s response to RelayMessage
        fn approve_transfer(origin, message_id: T::Hash) -> Result {
            let validator = ensure_signed(origin)?;
            ensure!(Self::bridge_is_operational(), "Bridge is not operational");
            Self::check_validator(validator.clone())?;

            let id = <TransferId<T>>::get(message_id);
            Self::_sign(validator, id)
        }

        // each validator calls it to add new validator
        fn add_validator(origin, address: T::AccountId) -> Result {
            let validator = ensure_signed(origin)?;
            Self::check_validator(validator.clone())?;

            ensure!(<ValidatorsCount<T>>::get() < 100_000, "Validators maximum reached.");
            let hash = ("add", &address).using_encoded(<T as system::Trait>::Hashing::hash);

            if !<ValidatorHistory<T>>::exists(hash) {
                let message = ValidatorMessage {
                    message_id: hash,
                    account: address,
                    action: Status::AddValidator,
                    status: Status::AddValidator,
                };
                <ValidatorHistory<T>>::insert(hash, message);
                Self::get_transfer_id_checked(hash, Kind::Validator)?;
            }

            let id = <TransferId<T>>::get(hash);
            Self::_sign(validator, id)
        }

        // each validator calls it to remove new validator
        fn remove_validator(origin, address: T::AccountId) -> Result {
            let validator = ensure_signed(origin)?;
            Self::check_validator(validator.clone())?;

            ensure!(<ValidatorsCount<T>>::get() > 1, "Can not remove last validator.");

            let hash = ("remove", &address).using_encoded(<T as system::Trait>::Hashing::hash);

            if !<ValidatorHistory<T>>::exists(hash) {
                let message = ValidatorMessage {
                    message_id: hash,
                    account: address,
                    action: Status::RemoveValidator,
                    status: Status::RemoveValidator,
                };
                <ValidatorHistory<T>>::insert(hash, message);
                Self::get_transfer_id_checked(hash, Kind::Validator)?;
            }

            let id = <TransferId<T>>::get(hash);
            Self::_sign(validator, id)
        }

        // each validator calls it to pause the bridge
        fn pause_bridge(origin) -> Result {
            let validator = ensure_signed(origin)?;
            Self::check_validator(validator.clone())?;

            ensure!(Self::bridge_is_operational(), "Bridge is not operational already");
            let hash = ("pause", T::BlockNumber::sa(0)).using_encoded(<T as system::Trait>::Hashing::hash);

            if !<BridgeMessages<T>>::exists(hash) {
                let message = BridgeMessage {
                    message_id: hash,
                    account: validator.clone(),
                    action: Status::PauseTheBridge,
                    status: Status::PauseTheBridge,
                };
                <BridgeMessages<T>>::insert(hash, message);
                Self::get_transfer_id_checked(hash, Kind::Bridge)?;
            }

            let id = <TransferId<T>>::get(hash);
            Self::_sign(validator, id)
        }

        // each validator calls it to resume the bridge
        fn resume_bridge(origin) -> Result {
            let validator = ensure_signed(origin)?;
            Self::check_validator(validator.clone())?;

            let hash = ("resume", T::BlockNumber::sa(0)).using_encoded(<T as system::Trait>::Hashing::hash);

            if !<BridgeMessages<T>>::exists(hash) {
                let message = BridgeMessage {
                    message_id: hash,
                    account: validator.clone(),
                    action: Status::ResumeTheBridge,
                    status: Status::ResumeTheBridge,
                };
                <BridgeMessages<T>>::insert(hash, message);
                Self::get_transfer_id_checked(hash, Kind::Bridge)?;
            }

            let id = <TransferId<T>>::get(hash);
            Self::_sign(validator, id)
        }

        //confirm burn from validator
        fn confirm_transfer(origin, message_id: T::Hash) -> Result {
            let validator = ensure_signed(origin)?;
            ensure!(Self::bridge_is_operational(), "Bridge is not operational");
            Self::check_validator(validator.clone())?;

            let id = <TransferId<T>>::get(message_id);

            let is_approved = <TransferMessages<T>>::get(message_id).status == Status::Approved ||
            <TransferMessages<T>>::get(message_id).status == Status::Confirmed;
            ensure!(is_approved, "This transfer must be approved first.");

            Self::update_status(message_id, Status::Confirmed, Kind::Transfer)?;
            Self::reopen_for_burn_confirmation(message_id)?;
            Self::_sign(validator, id)?;

            Ok(())
        }

        //cancel burn from validator
        fn cancel_transfer(origin, message_id: T::Hash) -> Result {
            let validator = ensure_signed(origin)?;
            ensure!(Self::bridge_is_operational(), "Bridge is not operational");
            Self::check_validator(validator)?;

            let mut message = <TransferMessages<T>>::get(message_id);
            message.status = Status::Canceled;

            <token::Module<T>>::unlock(&message.substrate_address, message.amount)?;
            <TransferMessages<T>>::insert(message_id, message);

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn _sign(validator: T::AccountId, transfer_id: ProposalId) -> Result {
        let mut transfer = <BridgeTransfers<T>>::get(transfer_id);

        let mut message = <TransferMessages<T>>::get(transfer.message_id);
        let mut limit_message = <LimitMessages<T>>::get(transfer.message_id);
        let mut validator_message = <ValidatorHistory<T>>::get(transfer.message_id);
        let mut bridge_message = <BridgeMessages<T>>::get(transfer.message_id);
        let voted = <ValidatorVotes<T>>::get((transfer_id, validator.clone()));
        ensure!(!voted, "This validator has already voted.");
        ensure!(transfer.open, "This transfer is not open");
        transfer.votes += 1;

        if Self::votes_are_enough(transfer.votes) {
            match message.status {
                Status::Confirmed => (), // if burn is confirmed
                _ => match transfer.kind {
                    Kind::Transfer => message.status = Status::Approved,
                    Kind::Limits => limit_message.status = Status::Approved,
                    Kind::Validator => validator_message.status = Status::Approved,
                    Kind::Bridge => bridge_message.status = Status::Approved,
                },
            }
            match transfer.kind {
                Kind::Transfer => Self::execute_transfer(message)?,
                Kind::Limits => Self::manage_limits(limit_message)?,
                Kind::Validator => Self::manage_validator(validator_message)?,
                Kind::Bridge => Self::manage_bridge(bridge_message)?,
            }
            transfer.open = false;
        } else {
            match message.status {
                Status::Confirmed => (),
                _ => Self::set_pending(transfer_id, transfer.kind.clone())?,
            };
        }

        <ValidatorVotes<T>>::mutate((transfer_id, validator), |a| *a = true);
        <BridgeTransfers<T>>::insert(transfer_id, transfer);

        Ok(())
    }

    ///ensure that such transfer exist
    fn get_transfer_id_checked(transfer_hash: T::Hash, kind: Kind) -> Result {
        if !<TransferId<T>>::exists(transfer_hash) {
            Self::create_transfer(transfer_hash, kind)?;
        }

        Ok(())
    }

    fn deposit(message: TransferMessage<T::AccountId, T::Hash>) -> Result {
        let to = message.substrate_address;

        <PendingMintCount<T>>::mutate(|c| *c -= 1);
        if !<DailyHolds<T>>::exists(&to) {
            <DailyHolds<T>>::insert(to.clone(), (T::BlockNumber::sa(0), message.message_id));
        }

        <token::Module<T>>::_mint(to, message.amount)?;

        Self::deposit_event(RawEvent::MintedMessage(message.message_id));
        Self::update_status(message.message_id, Status::Confirmed, Kind::Transfer)
    }

    fn withdraw(message: TransferMessage<T::AccountId, T::Hash>) -> Result {
        Self::check_daily_holds(message.clone())?;
        <PendingBurnCount<T>>::mutate(|c| *c -= 1);

        let to = message.eth_address;
        let from = message.substrate_address;
        Self::lock_for_burn(from.clone(), message.amount)?;
        Self::deposit_event(RawEvent::ApprovedRelayMessage(
            message.message_id,
            from,
            to,
            message.amount,
        ));
        Self::update_status(message.message_id, Status::Approved, Kind::Transfer)
    }
    fn pause_the_bridge(message: BridgeMessage<T::AccountId, T::Hash>) -> Result {
        <BridgeIsOperational<T>>::mutate(|x| *x = false);
        Self::update_status(message.message_id, Status::Confirmed, Kind::Bridge)
    }

    fn resume_the_bridge(message: BridgeMessage<T::AccountId, T::Hash>) -> Result {
        <BridgeIsOperational<T>>::mutate(|x| *x = true);
        Self::update_status(message.message_id, Status::Confirmed, Kind::Bridge)
    }

    fn _change_max_limit(message: LimitMessage<T::Hash>) -> Result {
        Self::check_limit(message.amount)?;
        <MaxLimit<T>>::put(message.amount);
        Self::update_status(message.message_id, Status::Confirmed, Kind::Limits)
    }

    fn _change_min_limit(message: LimitMessage<T::Hash>) -> Result {
        Self::check_limit(message.amount)?;
        <MinLimit<T>>::put(message.amount);
        Self::update_status(message.message_id, Status::Confirmed, Kind::Limits)
    }

    fn _change_pending_burn_limit(message: LimitMessage<T::Hash>) -> Result {
        <PendingBurnLimit<T>>::put(message.amount);
        Self::update_status(message.message_id, Status::Confirmed, Kind::Limits)
    }

    fn _change_pending_mint_limit(message: LimitMessage<T::Hash>) -> Result {
        <PendingMintLimit<T>>::put(message.amount);
        Self::update_status(message.message_id, Status::Confirmed, Kind::Limits)
    }

    /// add validator
    fn _add_validator(info: ValidatorMessage<T::AccountId, T::Hash>) -> Result {
        ensure!(
            <ValidatorsCount<T>>::get() < MAX_VALIDATORS,
            "Validators maximum reached."
        );
        <Validators<T>>::insert(info.account.clone(), true);
        <ValidatorAccounts<T>>::mutate(|v| v.retain(|x| *x != info.account));
        <ValidatorsCount<T>>::mutate(|x| *x += 1);
        Self::update_status(info.message_id, Status::Confirmed, Kind::Validator)
    }

    /// remove validator
    fn _remove_validator(info: ValidatorMessage<T::AccountId, T::Hash>) -> Result {
        ensure!(
            <ValidatorsCount<T>>::get() > 1,
            "Can not remove last validator."
        );
        <Validators<T>>::remove(info.account);
        <ValidatorsCount<T>>::mutate(|x| *x -= 1);
        <ValidatorHistory<T>>::remove(info.message_id);
        Ok(())
    }

    /// check votes validity
    fn votes_are_enough(votes: MemberId) -> bool {
        votes as f64 / Self::validators_count() as f64 >= 0.51
    }

    /// lock funds after set_transfer call
    fn lock_for_burn(account: T::AccountId, amount: TokenBalance) -> Result {
        <token::Module<T>>::lock(account, amount)?;

        Ok(())
    }

    fn execute_burn(message_id: T::Hash) -> Result {
        let message = <TransferMessages<T>>::get(message_id);
        let from = message.substrate_address.clone();
        let to = message.eth_address;

        <token::Module<T>>::unlock(&from, message.amount)?;
        <token::Module<T>>::_burn(from.clone(), message.amount)?;

        Self::deposit_event(RawEvent::BurnedMessage(
            message_id,
            from,
            to,
            message.amount,
        ));
        Ok(())
    }

    fn execute_transfer(message: TransferMessage<T::AccountId, T::Hash>) -> Result {
        match message.action {
            Status::Deposit => match message.status {
                Status::Approved => Self::deposit(message),
                _ => Err("Tried to deposit with non-supported status"),
            },
            Status::Withdraw => match message.status {
                Status::Confirmed => Self::execute_burn(message.message_id),
                Status::Approved => Self::withdraw(message),
                _ => Err("Tried to withdraw with non-supported status"),
            },
            _ => Err("Tried to execute transfer with non-supported status"),
        }
    }

    fn manage_validator(message: ValidatorMessage<T::AccountId, T::Hash>) -> Result {
        match message.action {
            Status::AddValidator => match message.status {
                Status::Approved => Self::_add_validator(message),
                _ => Err("Tried to add validator with non-supported status"),
            },
            Status::RemoveValidator => match message.status {
                Status::Approved => Self::_remove_validator(message),
                _ => Err("Tried to remove validator with non-supported status"),
            },
            _ => Err("Tried to manage validator with non-supported status"),
        }
    }

    fn manage_bridge(message: BridgeMessage<T::AccountId, T::Hash>) -> Result {
        match message.action {
            Status::PauseTheBridge => match message.status {
                Status::Approved => Self::pause_the_bridge(message),
                _ => Err("Tried to pause the bridge with non-supported status"),
            },
            Status::ResumeTheBridge => match message.status {
                Status::Approved => Self::resume_the_bridge(message),
                _ => Err("Tried to resume the bridge with non-supported status"),
            },
            _ => Err("Tried to manage bridge with non-supported status"),
        }
    }

    fn manage_limits(message: LimitMessage<T::Hash>) -> Result {
        match message.action {
            Status::ChangeMinTx => match message.status {
                Status::Approved => Self::_change_min_limit(message),
                _ => Err("Tried to pause the bridge with non-supported status"),
            },
            Status::ChangeMaxTx => match message.status {
                Status::Approved => Self::_change_max_limit(message),
                _ => Err("Tried to resume the bridge with non-supported status"),
            },
            Status::ChangePendingBurnLimit => match message.status {
                Status::Approved => Self::_change_pending_burn_limit(message),
                _ => Err("Tried to change pending burn limit with non-supported status"),
            },
            Status::ChangePendingMintLimit => match message.status {
                Status::Approved => Self::_change_pending_mint_limit(message),
                _ => Err("Tried to change pending mint limit with non-supported status"),
            },
            _ => Err("Tried to manage bridge with non-supported status"),
        }
    }

    fn create_transfer(transfer_hash: T::Hash, kind: Kind) -> Result {
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
            kind,
        };

        <BridgeTransfers<T>>::insert(transfer_id, transfer);
        <BridgeTransfersCount<T>>::mutate(|count| *count = new_bridge_transfers_count);
        <TransferId<T>>::insert(transfer_hash, transfer_id);
        <MessageId<T>>::insert(transfer_id, transfer_hash);

        Ok(())
    }
    fn set_pending(transfer_id: ProposalId, kind: Kind) -> Result {
        let message_id = <MessageId<T>>::get(transfer_id);
        match kind {
            Kind::Transfer => {
                let message = <TransferMessages<T>>::get(message_id);
                match message.action {
                    Status::Withdraw => <PendingBurnCount<T>>::mutate(|c| *c += 1),
                    Status::Deposit => <PendingMintCount<T>>::mutate(|c| *c += 1),
                    _ => (),
                }
            }
            _ => (),
        }
        Self::update_status(message_id, Status::Pending, kind)
    }

    fn update_status(id: T::Hash, status: Status, kind: Kind) -> Result {
        match kind {
            Kind::Transfer => {
                let mut message = <TransferMessages<T>>::get(id);
                message.status = status;
                <TransferMessages<T>>::insert(id, message);
            }
            Kind::Validator => {
                let mut message = <ValidatorHistory<T>>::get(id);
                message.status = status;
                <ValidatorHistory<T>>::insert(id, message);
            }
            Kind::Bridge => {
                let mut message = <BridgeMessages<T>>::get(id);
                message.status = status;
                <BridgeMessages<T>>::insert(id, message);
            }
            Kind::Limits => {
                let mut message = <LimitMessages<T>>::get(id);
                message.status = status;
                <LimitMessages<T>>::insert(id, message);
            }
        }
        Ok(())
    }

    // needed because @message_id will be the same as initial
    fn reopen_for_burn_confirmation(message_id: T::Hash) -> Result {
        let message = <TransferMessages<T>>::get(message_id);
        let transfer_id = <TransferId<T>>::get(message_id);
        let mut transfer = <BridgeTransfers<T>>::get(transfer_id);
        if !transfer.open && message.status == Status::Confirmed {
            transfer.votes = 0;
            transfer.open = true;
            <BridgeTransfers<T>>::insert(transfer_id, transfer);
            let validators = <ValidatorAccounts<T>>::get();
            validators
                .iter()
                .for_each(|a| <ValidatorVotes<T>>::insert((transfer_id, a.clone()), false));
        }
        Ok(())
    }
    fn check_validator(validator: T::AccountId) -> Result {
        let is_trusted = <Validators<T>>::exists(validator);
        ensure!(is_trusted, "Only validators can call this function");

        Ok(())
    }
    fn check_amount(amount: TokenBalance) -> Result {
        let token = <token::Module<T>>::token_default();
        let max = <MaxLimit<T>>::get() * 10u128.pow(token.decimals.into());
        let min = <MinLimit<T>>::get() * 10u128.pow(token.decimals.into());

        ensure!(
            amount > min,
            "Invalid amount for transaction. Reached minimum limit."
        );
        ensure!(
            amount < max,
            "Invalid amount for transaction. Reached maximum limit."
        );

        Ok(())
    }

    fn check_limit(amount: TokenBalance) -> Result {
        let max = u128::max_value();
        let min = u128::min_value();
        ensure!(amount < max, "Overflow setting limit");
        ensure!(amount > min, "Underflow setting limit");

        Ok(())
    }

    fn check_daily_holds(message: TransferMessage<T::AccountId, T::Hash>) -> Result {
        let from = message.substrate_address;
        let first_tx = <DailyHolds<T>>::get(from.clone());
        let daily_hold = T::BlockNumber::sa(DAY);
        let day_passed = first_tx.0 + daily_hold < T::BlockNumber::sa(0);

        if !day_passed {
            let account_balance = <token::Module<T>>::balance_of(from);
            // 75% of potentially really big numbers
            let allowed_amount = account_balance
                .checked_div(100)
                .expect("Failed to calculate allowed withdraw amount")
                .checked_mul(75)
                .expect("Failed to calculate allowed withdraw amount");

            if message.amount > allowed_amount {
                Self::update_status(message.message_id, Status::Canceled, Kind::Transfer)?;
                fail!("Cannot withdraw more that 75% of first day deposit.");
            }
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
    const ETH_MESSAGE_ID2: &[u8; 32] = b"0x5617yhk391571b5dc8230db92ba65b";
    const ETH_MESSAGE_ID3: &[u8; 32] = b"0x5617jdp391571b5dc8230db92ba65b";
    const ETH_ADDRESS: &[u8; 20] = b"0x00b46c2526ebb8f4c9";
    const V1: u64 = 1;
    const V2: u64 = 2;
    const V3: u64 = 3;
    const V4: u64 = 4;
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

        r.extend(
            GenesisConfig::<Test> {
                validators_count: 3u32,
                validator_accounts: vec![V1, V2, V3],
                pending_burn_limit: 2,
                pending_mint_limit: 2,
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
            let amount = 999 * 10u128.pow(18);

            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                message_id,
                eth_address,
                USER2,
                amount
            ));
            let mut message = BridgeModule::messages(message_id);
            assert_eq!(message.status, Status::Pending);

            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V1),
                message_id,
                eth_address,
                USER2,
                amount
            ));
            message = BridgeModule::messages(message_id);
            assert_eq!(message.status, Status::Confirmed);

            let transfer = BridgeModule::transfers(0);
            assert_eq!(transfer.open, false);

            assert_eq!(TokenModule::balance_of(USER2), amount);
            assert_eq!(TokenModule::total_supply(), amount);
        })
    }
    #[test]
    fn token_eth2sub_closed_transfer_fail() {
        with_externalities(&mut new_test_ext(), || {
            let message_id = H256::from(ETH_MESSAGE_ID);
            let eth_address = H160::from(ETH_ADDRESS);
            let amount = 999 * 10u128.pow(18);

            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                message_id,
                eth_address,
                USER2,
                amount
            ));
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V1),
                message_id,
                eth_address,
                USER2,
                amount
            ));
            assert_noop!(
                BridgeModule::multi_signed_mint(
                    Origin::signed(V3),
                    message_id,
                    eth_address,
                    USER2,
                    amount
                ),
                "This transfer is not open"
            );
            assert_eq!(TokenModule::balance_of(USER2), amount);
            assert_eq!(TokenModule::total_supply(), amount);
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
            let amount1 = 999 * 10u128.pow(18);
            let amount2 = 500 * 10u128.pow(18);

            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                eth_message_id,
                eth_address,
                USER2,
                amount1
            ));
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V1),
                eth_message_id,
                eth_address,
                USER2,
                amount1
            ));

            //substrate ----> ETH
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                eth_address,
                amount2
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
            assert_eq!(TokenModule::locked(USER2), amount2);
            assert_eq!(TokenModule::balance_of(USER2), amount1);
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
            //BurnedMessage(Hash, AccountId, H160, u64) event emitted
            let tokens_left = amount1 - amount2;
            assert_eq!(TokenModule::balance_of(USER2), tokens_left);
            assert_eq!(TokenModule::total_supply(), tokens_left);
        })
    }
    #[test]
    fn token_sub2eth_burn_fail_skip_approval() {
        with_externalities(&mut new_test_ext(), || {
            let eth_message_id = H256::from(ETH_MESSAGE_ID);
            let eth_address = H160::from(ETH_ADDRESS);
            let amount1 = 999 * 10u128.pow(18);
            let amount2 = 500 * 10u128.pow(18);
            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                eth_message_id,
                eth_address,
                USER2,
                amount1
            ));
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V1),
                eth_message_id,
                eth_address,
                USER2,
                amount1
            ));
            assert_eq!(TokenModule::balance_of(USER2), amount1);
            assert_eq!(TokenModule::total_supply(), amount1);

            //substrate ----> ETH
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                eth_address,
                amount2
            ));
            //RelayMessage(message_id) event emitted

            let sub_message_id = BridgeModule::message_id_by_transfer_id(1);
            let message = BridgeModule::messages(sub_message_id);
            assert_eq!(message.status, Status::Withdraw);

            assert_eq!(TokenModule::locked(USER2), 0);
            // lets say validators blacked out and we
            // try to confirm without approval anyway
            assert_noop!(
                BridgeModule::confirm_transfer(Origin::signed(V1), sub_message_id),
                "This transfer must be approved first."
            );
        })
    }
    #[test]
    fn add_validator_should_work() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(BridgeModule::add_validator(Origin::signed(V2), V4));
            let id = BridgeModule::message_id_by_transfer_id(0);
            let mut message = BridgeModule::validator_history(id);
            assert_eq!(message.status, Status::Pending);

            assert_ok!(BridgeModule::add_validator(Origin::signed(V1), V4));
            message = BridgeModule::validator_history(id);
            assert_eq!(message.status, Status::Confirmed);
            assert_eq!(BridgeModule::validators_count(), 4);
        })
    }
    #[test]
    fn remove_validator_should_work() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(BridgeModule::remove_validator(Origin::signed(V2), V3));
            let id = BridgeModule::message_id_by_transfer_id(0);
            let mut message = BridgeModule::validator_history(id);
            assert_eq!(message.status, Status::Pending);

            assert_ok!(BridgeModule::remove_validator(Origin::signed(V1), V3));
            message = BridgeModule::validator_history(id);
            assert_eq!(message.status, Status::Revoked);
            assert_eq!(BridgeModule::validators_count(), 2);
        })
    }
    #[test]
    fn remove_last_validator_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(BridgeModule::remove_validator(Origin::signed(V2), V3));
            assert_ok!(BridgeModule::remove_validator(Origin::signed(V1), V3));
            assert_eq!(BridgeModule::validators_count(), 2);

            //TODO: deal with two validators corner case
            assert_ok!(BridgeModule::remove_validator(Origin::signed(V1), V2));
            assert_ok!(BridgeModule::remove_validator(Origin::signed(V2), V2));
            // ^ this guy probably will not sign his removal ^

            assert_eq!(BridgeModule::validators_count(), 1);
            // assert_noop BUG: fails through different root hashes
            // assert_ok fails with corect error but the noop below fails with different root hashes
            // assert_noop!(BridgeModule::remove_validator(Origin::signed(V1), V1), "Cant remove last validator");
        })
    }
    #[test]
    fn pause_the_bridge_should_work() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V2)));

            assert_eq!(BridgeModule::bridge_transfers_count(), 1);
            assert_eq!(BridgeModule::bridge_is_operational(), true);
            let id = BridgeModule::message_id_by_transfer_id(0);
            let mut message = BridgeModule::bridge_messages(id);
            assert_eq!(message.status, Status::Pending);

            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V1)));
            assert_eq!(BridgeModule::bridge_is_operational(), false);
            message = BridgeModule::bridge_messages(id);
            assert_eq!(message.status, Status::Confirmed);
        })
    }
    #[test]
    fn extrinsics_restricted_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            let eth_message_id = H256::from(ETH_MESSAGE_ID);
            let eth_address = H160::from(ETH_ADDRESS);

            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V2)));
            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V1)));

            // substrate <-- Ethereum
            assert_noop!(
                BridgeModule::multi_signed_mint(
                    Origin::signed(V2),
                    eth_message_id,
                    eth_address,
                    USER2,
                    1000
                ),
                "Bridge is not operational"
            );
        })
    }
    #[test]
    fn double_pause_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(BridgeModule::bridge_is_operational(), true);
            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V2)));
            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V1)));
            assert_eq!(BridgeModule::bridge_is_operational(), false);
            assert_noop!(
                BridgeModule::pause_bridge(Origin::signed(V1)),
                "Bridge is not operational already"
            );
        })
    }
    #[test]
    fn pause_and_resume_the_bridge_should_work() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(BridgeModule::bridge_is_operational(), true);
            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V2)));
            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V1)));
            assert_eq!(BridgeModule::bridge_is_operational(), false);
            assert_ok!(BridgeModule::resume_bridge(Origin::signed(V1)));
            assert_ok!(BridgeModule::resume_bridge(Origin::signed(V2)));
            assert_eq!(BridgeModule::bridge_is_operational(), true);
        })
    }
    #[test]
    fn double_vote_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(BridgeModule::bridge_is_operational(), true);
            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V2)));
            assert_noop!(
                BridgeModule::pause_bridge(Origin::signed(V2)),
                "This validator has already voted."
            );
        })
    }
    #[test]
    fn change_min_limit_should_work() {
        with_externalities(&mut new_test_ext(), || {
            const LESS_THAN_MINIMUM: u128 = 5;
            let message_id = H256::from(ETH_MESSAGE_ID);

            assert_eq!(BridgeModule::min_tx_limit(), 10);
            assert_ok!(BridgeModule::change_min_limit(
                Origin::signed(V2),
                message_id,
                LESS_THAN_MINIMUM
            ));
            assert_ok!(BridgeModule::change_min_limit(
                Origin::signed(V1),
                message_id,
                LESS_THAN_MINIMUM
            ));
            assert_eq!(BridgeModule::min_tx_limit(), LESS_THAN_MINIMUM);
        })
    }
    #[test]
    fn change_max_limit_should_work() {
        with_externalities(&mut new_test_ext(), || {
            const MORE_THAN_MAX: u128 = 10000;
            let message_id = H256::from(ETH_MESSAGE_ID);

            assert_eq!(BridgeModule::max_tx_limit(), 1000);
            assert_ok!(BridgeModule::change_max_limit(
                Origin::signed(V2),
                message_id,
                MORE_THAN_MAX
            ));
            assert_ok!(BridgeModule::change_max_limit(
                Origin::signed(V1),
                message_id,
                MORE_THAN_MAX
            ));
            assert_eq!(BridgeModule::max_tx_limit(), MORE_THAN_MAX);
        })
    }
    #[test]
    fn change_min_limit_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            const LESS_THAN_MINIMUM: u128 = u128::min_value();
            let message_id = H256::from(ETH_MESSAGE_ID);

            assert_eq!(BridgeModule::min_tx_limit(), 10);
            assert_noop!(
                BridgeModule::change_min_limit(Origin::signed(V2), message_id, LESS_THAN_MINIMUM),
                "Underflow setting limit"
            );
        })
    }
    #[test]
    fn change_max_limit_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            const MORE_THAN_MAX: u128 = u128::max_value();
            let message_id = H256::from(ETH_MESSAGE_ID);

            assert_eq!(BridgeModule::max_tx_limit(), 1000);
            assert_noop!(
                BridgeModule::change_max_limit(Origin::signed(V2), message_id, MORE_THAN_MAX),
                "Overflow setting limit"
            );
        })
    }
    #[test]
    fn instant_withdraw_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            let eth_message_id = H256::from(ETH_MESSAGE_ID);
            let eth_address = H160::from(ETH_ADDRESS);
            let amount1 = 999 * 10u128.pow(18);
            let amount2 = 900 * 10u128.pow(18);

            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                eth_message_id,
                eth_address,
                USER2,
                amount1
            ));
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V1),
                eth_message_id,
                eth_address,
                USER2,
                amount1
            ));

            //substrate ----> ETH
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                eth_address,
                amount2
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

            // assert_noop BUG: fails through different root hashes
            // assert_noop!(
            //     BridgeModule::approve_transfer(Origin::signed(V2), sub_message_id),
            //     "Cannot withdraw more that 75% of first day deposit."
            // );
            // signs the transfer, but fails further and marks message as Canceled
            let _ = BridgeModule::approve_transfer(Origin::signed(V2), sub_message_id);

            message = get_message();
            assert_eq!(message.status, Status::Canceled);
        })
    }
    #[test]
    fn change_pending_burn_limit_should_work() {
        with_externalities(&mut new_test_ext(), || {
            let eth_message_id = H256::from(ETH_MESSAGE_ID);
            const AMOUNT1: u128 = 5;

            assert_eq!(BridgeModule::pending_burn_limit(), 2);
            assert_ok!(BridgeModule::set_pending_burn_limit(
                Origin::signed(V2),
                eth_message_id,
                AMOUNT1
            ));
            assert_ok!(BridgeModule::set_pending_burn_limit(
                Origin::signed(V1),
                eth_message_id,
                AMOUNT1
            ));

            assert_eq!(BridgeModule::pending_burn_limit(), 5);
        })
    }
    #[test]
    fn change_pending_mint_limit_should_work() {
        with_externalities(&mut new_test_ext(), || {
            let eth_message_id = H256::from(ETH_MESSAGE_ID);
            const AMOUNT1: u128 = 5;

            assert_eq!(BridgeModule::pending_mint_limit(), 2);
            assert_ok!(BridgeModule::set_pending_mint_limit(
                Origin::signed(V2),
                eth_message_id,
                AMOUNT1
            ));
            assert_ok!(BridgeModule::set_pending_mint_limit(
                Origin::signed(V1),
                eth_message_id,
                AMOUNT1
            ));

            assert_eq!(BridgeModule::pending_mint_limit(), 5);
        })
    }
    #[test]
    fn pending_burn_limit_should_work() {
        with_externalities(&mut new_test_ext(), || {
            let eth_message_id = H256::from(ETH_MESSAGE_ID);
            let eth_address = H160::from(ETH_ADDRESS);
            let amount1 = 999 * 10u128.pow(18);
            let amount2 = 900 * 10u128.pow(18);

            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                eth_message_id,
                eth_address,
                USER2,
                amount1
            ));
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V1),
                eth_message_id,
                eth_address,
                USER2,
                amount1
            ));

            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                eth_address,
                amount1
            ));
            let sub_message_id = BridgeModule::message_id_by_transfer_id(1);
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V1),
                sub_message_id
            ));

            assert_eq!(BridgeModule::pending_burn_count(), 1);
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                eth_address,
                amount2
            ));

            let sub_message_id2 = BridgeModule::message_id_by_transfer_id(2);
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V2),
                sub_message_id2
            ));

            assert_eq!(BridgeModule::pending_burn_count(), 2);
            assert_noop!(
                BridgeModule::set_transfer(Origin::signed(USER2), eth_address, amount1 - amount2),
                "Too many pending burn transactions."
            );
        })
    }
    #[test]
    fn pending_mint_limit_should_work() {
        with_externalities(&mut new_test_ext(), || {
            let eth_message_id = H256::from(ETH_MESSAGE_ID);
            let eth_message_id2 = H256::from(ETH_MESSAGE_ID2);
            let eth_message_id3 = H256::from(ETH_MESSAGE_ID3);
            let eth_address = H160::from(ETH_ADDRESS);
            let amount1 = 999 * 10u128.pow(18);
            let amount2 = 900 * 10u128.pow(18);

            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                eth_message_id,
                eth_address,
                USER2,
                amount1
            ));

            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                eth_message_id2,
                eth_address,
                USER2,
                amount2
            ));

            //substrate <----- ETH
            assert_noop!(
                BridgeModule::multi_signed_mint(
                    Origin::signed(V2),
                    eth_message_id3,
                    eth_address,
                    USER2,
                    amount1 - amount2
                ),
                "Too many pending mint transactions."
            );
        })
    }
}

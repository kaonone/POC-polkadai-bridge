use support::{decl_event, decl_module, decl_storage, dispatch::Result, StorageValue};
use system::ensure_signed;

use rstd::prelude::Vec;

use crate::types::{DaoId, Days, Rate};

/// The module's configuration trait.
pub trait Trait: balances::Trait + system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as marketplace {
        Something get(something): Option<u64>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        fn make_investment(origin, proposal_id: u64) -> Result {
            let who = ensure_signed(origin)?;

            <Something<T>>::put(proposal_id);

            Self::deposit_event(RawEvent::NewInvsetment(proposal_id, who));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn propose_to_investment(
        dao_id: DaoId,
        description: Vec<u8>,
        days: Days,
        rate: Rate,
        value: T::Balance,
    ) -> Result {
        Self::deposit_event(RawEvent::ProposeToInvestment(
            dao_id,
            description,
            days,
            rate,
            value,
        ));
        Ok(())
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = <T as balances::Trait>::Balance,
    {
        NewInvsetment(u64, AccountId),
        ProposeToInvestment(DaoId, Vec<u8>, Days, Rate, Balance),
    }
);

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use support::{assert_ok, impl_outer_origin};

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
    impl Trait for Test {
        type Event = ();
    }
    type Marketplace = Module<Test>;

    const DAO_DESC: &[u8; 10] = b"Desc-1234_";

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .0
            .into()
    }

    #[test]
    fn make_investment_should_work() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(Marketplace::make_investment(Origin::signed(1), 42));
            assert_eq!(Marketplace::something(), Some(42));
        });
    }

    #[test]
    fn propose_to_investment_should_work() {
        const DAO_ID: DaoId = 0;
        const DAYS: Days = 181;
        const RATE: Rate = 1000;
        const VALUE: u128 = 42;

        with_externalities(&mut new_test_ext(), || {
            assert_ok!(Marketplace::propose_to_investment(
                DAO_ID,
                DAO_DESC.to_vec(),
                DAYS,
                RATE,
                VALUE
            ));
        });
    }
}

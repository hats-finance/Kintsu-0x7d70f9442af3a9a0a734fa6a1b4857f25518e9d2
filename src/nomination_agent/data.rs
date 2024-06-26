use ink::prelude::vec::Vec;
use ink::primitives::AccountId;

#[allow(dead_code)]
#[derive(scale::Encode)]
pub enum MultiAddress<AccountId, AccountIndex> {
    // It's an account ID (pubkey).
    Id(AccountId),
    // It's an account index.
    Index(#[codec(compact)] AccountIndex),
    // It's some arbitrary raw bytes.
    Raw(Vec<u8>),
    // It's a 32 byte representation.
    Address32([u8; 32]),
    // Its a 20 byte representation.
    Address20([u8; 20]),
}
#[derive(scale::Encode)]
pub enum BondExtra {
    FreeBalance { balance: u128 },
}
#[derive(scale::Encode)]
pub enum NominationCall {
    #[codec(index = 0)]
    Join {
        #[codec(compact)]
        amount: u128,
        pool_id: u32,
    },
    #[codec(index = 1)]
    BondExtra { extra: BondExtra },
    #[codec(index = 2)]
    ClaimPayout {},
    #[codec(index = 3)]
    Unbond {
        member_account: MultiAddress<AccountId, ()>,
        #[codec(compact)]
        unbonding_points: u128,
    },
    #[codec(index = 5)]
    WithdrawUnbonded {
        member_account: MultiAddress<AccountId, ()>,
        num_slashing_spans: u32,
    },
}
#[derive(scale::Encode)]
pub enum RuntimeCall {
    #[codec(index = 19)]
    NominationPools(NominationCall),
}

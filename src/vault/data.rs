use crate::nomination_agent_utils::{
    call_compound,
    call_deposit,
    call_unbond,
    call_withdraw_unbonded,
    query_staked_value,
    RuntimeError,
};
use ink::{
    env::{
        debug_println,
        DefaultEnvironment,
        Environment,
    },
    prelude::{string::String, vec::Vec},
    primitives::AccountId,
    storage::Mapping,
};
use psp22::PSP22Error;
use registry::{registry::Agent, RegistryRef};
type Balance = <DefaultEnvironment as Environment>::Balance;
type Timestamp = u64;

pub const BIPS: u16 = 10000;
pub const DAY: u64 = 86400 * 1000;
pub const YEAR: u64 = DAY * 365;

#[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
pub struct UnlockRequest {
    pub creation_time: Timestamp,
    pub share_amount: Balance,
    pub batch_id: u64,
}

#[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
pub struct UnlockRequestBatch {
    pub total_shares: Balance,
    pub value_at_redemption: Option<Balance>,
    pub redemption_timestamp: Option<Timestamp>,
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum VaultError {
    InvalidPercent,
    InvalidBatchUnlockRequest,
    InvalidUserUnlockRequest,
    CooldownPeriod,
    InvalidPermissions,
    NoChange,
    ZeroDepositing,
    ZeroUnbonding,
    ZeroTotalWeight,
    ZeroCompounding,
    MinimumStake,
    /// An interaction with ink! environment has failed
    // NOTE: We're representing the `ink::env::Error` as `String` b/c the
    // type does not have Encode/Decode implemented.
    InkEnvError(String),
    InternalError(RuntimeError),
    TokenError(PSP22Error),
    InternalTokenError,
}

#[ink::storage_item]
#[derive(Debug)]
pub struct VaultData {
    /// account that can withdraw fees, set minimum stake, and upgrade the Vault
    pub role_owner: AccountId,
    /// account that can adjust fees
    pub role_adjust_fee: AccountId,
    /// account that can change the `role_adjust_fee` account
    pub role_adjust_fee_admin: AccountId,
    /// contract creation block timestamp
    pub creation_time: Timestamp,

    /// total AZERO staked excluding AZERO being unbonded
    pub total_pooled: Balance,
    /// total sAZERO minted
    pub total_shares_minted: Balance,
    /// rolling accumulator of inflation fees (sAZERO shares) that can be minted and claimed by owner
    pub total_shares_virtual: Balance,
    /// minimum amount of AZERO a user needs to stake
    pub minimum_stake: Balance,

    /// record of all batched unlock requests indexed by batch id
    pub batch_unlock_requests: Mapping<u64, UnlockRequestBatch>,
    /// record of each user's unlock requests indexed by user AccountId
    pub user_unlock_requests: Mapping<AccountId, Vec<UnlockRequest>>,

    /// time required after submitting a batch unlock request until AZERO is withdraw-able
    pub cooldown_period: u64,
    /// minimum time between creating batch unlock requests to prevent excess request errors
    pub batch_interval_delay: u64,

    /// last update time of claimable fees variable only modified by stake, redeem, withdraw_fees, and adjust_fee
    pub last_fee_update: Timestamp,
    /// annualized fee percentage expressed in basis points
    pub fee_percentage: u16,
    /// compounding incentive percentage expressed in basis points
    pub incentive_percentage: u16,

    /// token contract used for representing protocol staked AZERO ownership
    pub shares_contract: AccountId,
    /// registry contract used for tracking nominator pools and weights
    pub registry_contract: RegistryRef,
}

impl VaultData {
    pub fn new(
        admin: AccountId,
        shares_contract_: AccountId,
        registry_ref: RegistryRef,
        current_time: Timestamp,
        era: u64,
    ) -> VaultData {
        VaultData {
            role_owner: admin,
            role_adjust_fee: admin,
            role_adjust_fee_admin: admin,
            creation_time: current_time,
            total_pooled: 0,
            total_shares_minted: 0,
            total_shares_virtual: 0,
            minimum_stake: 0,
            batch_unlock_requests: Mapping::default(),
            user_unlock_requests: Mapping::default(),
            cooldown_period: era * 14,
            batch_interval_delay: era * 2,
            last_fee_update: current_time,
            fee_percentage: 2_00, // 2.00%
            incentive_percentage: 0_05, // 0.05%
            shares_contract: shares_contract_,
            registry_contract: registry_ref,
        }
    }

    pub fn get_batch_unlock_id(&self, time: Timestamp) -> u64 {
        (time - self.creation_time) / self.batch_interval_delay
    }

    /// Calculates differences between current staked amounts and optimal staked amounts
    ///
    /// # Returns
    ///
    /// `total_staked` - Total AZERO staked across all agents
    /// `pos_diff` - Total positive difference; zero indicates no over-allocations
    /// `neg_diff` - Total negative difference; zero indicates no under-allocations
    /// `stakes` - Amount of AZERO staked in each agent
    /// `imbalances` - Deltas between the staked value and optimal value in each agent
    ///                Positive values indicate an over-allocation
    ///                Negative values indicate an under-allocation
    pub fn get_weight_imbalances(
        &self,
        agents: &Vec<Agent>,
        total_weight: u64,
        total_pooled: u128,
    ) -> (u128, u128, u128, Vec<u128>, Vec<i128>) {
        let mut total_staked = 0_u128;
        let mut pos_diff = 0_u128;
        let mut neg_diff = 0_u128;
        let mut stakes = Vec::new();
        let mut imbalances = Vec::new();

        for a in agents.into_iter() {
            let staked_amount_current = query_staked_value(a.address) as i128;
            let staked_amount_optimal = if total_weight > 0 {
                ((a.weight as u128 * total_pooled) / total_weight as u128) as i128
            } else {
                0
            };
            let diff = staked_amount_current - staked_amount_optimal;
            if diff > 0 {
                pos_diff += diff as u128;
            } else if diff < 0 {
                neg_diff += -diff as u128;
            }
            total_staked += staked_amount_current as u128;
            stakes.push(staked_amount_current as u128);
            imbalances.push(diff);
        }

        (total_staked, pos_diff, neg_diff, stakes, imbalances)
    }

    /// Deposits a given amount to nominator agents splitting deposits by nominator weights and stake imbalances
    ///
    /// Uses a weighting algorithm that prioritizes negatively imbalanced (under-allocated) pools.
    /// Phase1: The amount is split among negatively imbalanced nodes according to their proportion of the total imbalance.
    /// Phase2: If the deposit amount is more than the negative imbalance, the remainder is split according to nominator weight proportions.
    pub fn delegate_bonding(&mut self, azero: Balance) -> Result<(), VaultError> {
        let (total_weight, agents) = self.registry_contract.get_agents();

        if total_weight == 0 {
            return Err(VaultError::ZeroTotalWeight);
        }

        let new_total_pooled = self.total_pooled + azero;

        let (_total_staked, _pos_diff, neg_diff, _stakes, imbalances) = self
            .get_weight_imbalances(&agents, total_weight, new_total_pooled);

        // Amount to distribute to under-allocated agents
        let phase1 = if azero < neg_diff { azero } else { neg_diff };

        // Remaining amount to distribute equitably to all agents
        let phase2 = azero - phase1;

        let n = agents.len();
        let mut deposit_amounts: Vec<Balance> = Vec::with_capacity(n);
        let mut deposit_summation: Balance = 0;

        for i in 0..n {
            // Distribute to under-allocated agents
            // Weighted by agent imbalance
            let phase1_amount = if imbalances[i] < 0 {
                phase1 * (-imbalances[i] as u128) / neg_diff
            } else {
                0
            };

            // Distribute remaining amount equitably to all agents
            // Weighted by agent weight
            let phase2_amount = if phase2 > 0 {
                phase2 * (agents[i].weight as u128) / (total_weight as u128)
            } else {
                0
            };

            let deposit_amount = phase1_amount + phase2_amount;
            deposit_amounts.push(deposit_amount);
            deposit_summation += deposit_amount;
        }

        if deposit_summation == 0 {
            return Err(VaultError::ZeroDepositing);
        }

        let dust = azero - deposit_summation;
        debug_println!("Dust: {}", dust);

        // Allocate dust
        // Prioritizes agents added earlier in the registry
        // Fully allocates dust to the first agent which is receiving a deposit
        if dust > 0 {
            for i in 0..n {
                if deposit_amounts[i] > 0 {
                    debug_println!("Allocating {} dust to agent #{}", dust, i);
                    deposit_amounts[i] += dust;
                    break;
                }
            }
        }

        // Deposit
        for (i, a) in agents.iter().enumerate() {
            let deposit_amount = deposit_amounts[i];
            if deposit_amount > 0 {
                debug_println!("Depositing {} into agent #{}", deposit_amount, i);
                if let Err(e) = call_deposit(a.address, deposit_amount) {
                    return Err(VaultError::InternalError(e));
                }
            }
        }

        self.total_pooled = new_total_pooled;

        Ok(())
    }

    /// Unlocks a given amount of staked AZERO from the nominator pools
    ///
    /// Uses a weighting algorithm that prioritizes positively imbalanced (over-allocated) pools.
    /// Phase1: The amount is split among positively imbalanced nodes according to their proportion of the total imbalance.
    /// Phase2: If the unlock amount is more than the positive imbalance, the remainder is split according to nominator stake proportions.
    pub fn delegate_unbonding(&mut self, azero: Balance) -> Result<(), VaultError> {
        let (total_weight, agents) = self.registry_contract.get_agents();

        let new_total_pooled = self.total_pooled - azero;

        let (total_staked, pos_diff, _neg_diff, stakes, imbalances) = self
            .get_weight_imbalances(&agents, total_weight, new_total_pooled);

        // Amount to withdraw from over-allocated agents
        let phase1 = if azero < pos_diff { azero } else { pos_diff };

        // Remaining amount to withdraw equitably from all agents
        let phase2 = azero - phase1;

        let total_staked_after_phase1 = total_staked - phase1;

        let n = agents.len();
        let mut unbond_amounts: Vec<Balance> = Vec::with_capacity(n);
        let mut unbond_summation: Balance = 0;

        for i in 0..n {
            // Unbond from over-allocated agents
            // Weighted by agent imbalance
            let phase1_amount = if imbalances[i] > 0 {
                phase1 * (imbalances[i] as u128) / pos_diff
            } else {
                0
            };

            // Unbond remaining amount equitably from all agents
            // Weighted by agent remaining stake
            let phase2_amount = if phase2 > 0 {
                phase2 * (stakes[i] - phase1_amount) / total_staked_after_phase1
            } else {
                0
            };

            let unbond_amount = phase1_amount + phase2_amount;
            unbond_amounts.push(unbond_amount);
            unbond_summation += unbond_amount;
        }

        if unbond_summation == 0 {
            return Err(VaultError::ZeroUnbonding);
        }

        let mut dust = azero - unbond_summation;
        debug_println!("Dust: {}", dust);

        // Allocate dust
        // Prioritizes agents added earlier in the registry
        // Allocates dust to agents with surplus bonded AZERO
        // Splits dust across agents when first agent surplus is not sufficient
        if dust > 0 {
            for i in 0..n {
                if stakes[i] > unbond_amounts[i] {
                    let surplus = stakes[i] - unbond_amounts[i];
                    if dust > surplus {
                        debug_println!("Allocating {} dust to agent #{}", surplus, i);
                        unbond_amounts[i] += surplus;
                        dust -= surplus;
                    } else {
                        debug_println!("Allocating {} dust to agent #{}", dust, i);
                        unbond_amounts[i] += dust;
                        break;
                    }
                }
            }
        }

        // Unbond
        for (i, a) in agents.iter().enumerate() {
            let unbond_amount = unbond_amounts[i];
            if unbond_amount > 0 {
                debug_println!("Unbonding {} from agent #{}", unbond_amount, i);
                if let Err(e) = call_unbond(a.address, unbond_amount) {
                    return Err(VaultError::InternalError(e));
                }
            }
        }

        self.total_pooled = new_total_pooled;

        Ok(())
    }

    /// Claim all unbonded AZERO from the agents looping over each nominator pool
    pub fn delegate_withdraw_unbonded(&self) -> Result<(), VaultError> {
        let (_total_weight, agents) = self.registry_contract.get_agents();

        for a in agents.into_iter() {
            if let Err(e) = call_withdraw_unbonded(a.address) {
                return Err(VaultError::InternalError(e));
            }
        }

        Ok(())
    }

    /// Claim payouts and re-bond AZERO from the agents looping over each nominator pool
    ///
    /// # Returns
    ///
    /// `total_compounded` - Total AZERO compounded across all agents
    /// `total_incentive` - Total AZERO incentive from all agents
    pub fn delegate_compound(&mut self) -> Result<(Balance, Balance), VaultError> {
        let (_total_weight, agents) = self.registry_contract.get_agents();

        let mut total_compounded = 0;
        let mut total_incentive = 0;

        let incentive_percentage_ = self.incentive_percentage; // shadow

        for (i, a) in agents.into_iter().enumerate() {
            match call_compound(a.address, incentive_percentage_) {
                Ok((compound_amount, incentive_amount)) => {
                    debug_println!("Compounded {} to agent #{}", compound_amount, i);
                    total_compounded += compound_amount;
                    total_incentive += incentive_amount;
                },
                Err(e) => return Err(VaultError::InternalError(e)),
            }
        }

        if total_compounded == 0 {
            return Err(VaultError::ZeroCompounding);
        }

        self.total_pooled += total_compounded;

        Ok((total_compounded, total_incentive))
    }

    /// Calculates summation of fees from last update until now
    /// Must be called before changing: `total_shares_minted`, `fee_percentage`
    /// Must be called before calculating redemption ratio via: `get_shares_from_azero()` and `get_azero_from_shares()`
    pub fn update_fees(&mut self, current_time: Timestamp) {
        // Time since last update
        let time = current_time - self.last_fee_update;

        // Calculate fee accumulation since last update
        if time > 0 {
            let virtual_shares = self.total_shares_minted * self.fee_percentage as u128 / BIPS as u128;
            let time_weighted_virtual_shares = virtual_shares * time as u128 / YEAR as u128;

            self.total_shares_virtual += time_weighted_virtual_shares;
            self.last_fee_update = current_time;
        }
    }

    /// Returns the virtual shares that will exist at the given time
    pub fn get_virtual_shares_at_time(&self, current_time: Timestamp) -> Balance {
        // Time since last update
        let time = current_time - self.last_fee_update;

        if time > 0 {
            // Calculate fee accumulation since last update
            let virtual_shares = self.total_shares_minted * self.fee_percentage as u128 / BIPS as u128;
            let time_weighted_virtual_shares = virtual_shares * time as u128 / YEAR as u128;
            self.total_shares_virtual + time_weighted_virtual_shares
        } else {
            // No additional fee accumulation is required
            self.total_shares_virtual
        }
    }
}

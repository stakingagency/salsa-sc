#![no_std]

use crate::constants::*;

multiversx_sc::imports!();

pub mod storage;
pub mod logic;
pub mod state;
mod constants;
pub mod view;
mod proxy;

#[multiversx_sc::contract]
pub trait OneDexMock:
    storage::common_storage::CommonStorageModule
    + storage::pair_storage::PairStorageModule
    + logic::common::CommonLogicModule
    + logic::pair::PairLogicModule
    + logic::liquidity::LiquidityLogicModule
    + logic::swap::SwapLogicModule
    + logic::amm::AmmLogicModule
    + view::ViewModule
{
    #[init]
    fn init(&self,
        wegld_id: TokenIdentifier,
        wrap_address: ManagedAddress,
    ) {
        self.wegld_id().set_if_empty(wegld_id);
        self.total_fee_percent().set(TOTAL_FEE);
        self.special_fee_percent().set(SPECIAL_FEE);
        self.staking_reward_fee_percent().set(STAKING_FEE);
        let caller = self.blockchain().get_caller();
        self.treasury_address().set(caller.clone());
        self.staking_reward_address().set(caller.clone());
        self.burner_address().set(caller);
        self.unwrap_address().set(wrap_address);
        self.registering_cost().set(BigUint::from(REGISTERING_COST));
    }

    #[only_owner]
    #[endpoint(addMainPair)]
    fn add_main_pair(
        &self,
        token: TokenIdentifier,
    ) {
        require!(
            !self.main_pair_tokens().contains(&token),
            "token already exists",
        );

        self.main_pair_tokens().insert(token);
    }
}

multiversx_sc::imports!();

use crate::common::config::State;
use crate::common::errors::*;
use crate::common::storage_cache::StorageCache;
use crate::exchanges::onedex_cache::OnedexCache;
use crate::exchanges::xexchange_cache::XexchangeCache;
use crate::proxies::onedex_proxy::ProxyTrait as _;
use crate::proxies::xstake_proxy::{self, Stake, UserStake};

#[multiversx_sc::module]
pub trait XStakeModule:
    crate::common::config::ConfigModule
    + crate::helpers::HelpersModule
    + crate::exchanges::onedex::OnedexModule
    + crate::exchanges::xexchange::XexchangeModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[only_owner]
    #[endpoint(setXStakeActive)]
    fn set_xstake_active(&self) {
        require!(
            self.lp_state().get() == State::Active,
            ERROR_LP_MODULE_INACTIVE
        );

        require!(
            !self.xstake_sc().is_empty(),
            ERROR_XSTAKE_SC,
        );

        require!(
            !self.xstake_onedex_id().is_empty() && !self.xstake_xexchange_id().is_empty(),
            ERROR_XSTAKE_IDS_NOT_SET
        );

        self.xstake_state().set(State::Active);
    }

    #[only_owner]
    #[endpoint(setXStakeInactive)]
    fn set_xstake_inactive(&self) {
        let onedex_xstake_id = self.xstake_onedex_id().get();
        let onedex_stake = self.get_xstake(onedex_xstake_id);
        let onedex_user_stake =
            self.get_user_xstake(onedex_xstake_id, self.blockchain().get_sc_address());
        let onedex_staked = onedex_user_stake.staked.get(0).clone_value();
        if onedex_staked > 0 {
            self.remove_xstake(onedex_xstake_id, onedex_staked, onedex_stake.stake_tokens.get(0).clone_value());
        }
        let mut storage_cache = StorageCache::new(self);
        self.take_xstake_profit(onedex_xstake_id, &mut storage_cache);

        let xexchange_xstake_id = self.xstake_xexchange_id().get();
        let xexchange_stake = self.get_xstake(xexchange_xstake_id);
        let xexchange_user_stake =
            self.get_user_xstake(xexchange_xstake_id, self.blockchain().get_sc_address());
        let xexchange_staked = xexchange_user_stake.staked.get(0).clone_value();
        if xexchange_staked > 0 {
            self.remove_xstake(xexchange_xstake_id, xexchange_staked, xexchange_stake.stake_tokens.get(0).clone_value());
        }
        self.take_xstake_profit(xexchange_xstake_id, &mut storage_cache);

        self.xstake_state().set(State::Inactive);
    }

    #[inline]
    fn is_xstake_active(&self) -> bool {
        let xstake = self.xstake_state().get();
        xstake == State::Active
    }

    #[view(getXStakeState)]
    #[storage_mapper("xstake_state")]
    fn xstake_state(&self) -> SingleValueMapper<State>;

    #[storage_mapper("xstake_sc")]
    fn xstake_sc(&self) -> SingleValueMapper<ManagedAddress>;

    #[only_owner]
    #[endpoint(setXStakeSC)]
    fn set_xstake_sc(&self, address: ManagedAddress) {
        let state: xstake_proxy::State = self.xstake_proxy_obj()
            .contract(address.clone())
            .state()
            .execute_on_dest_context();

        require!(state == xstake_proxy::State::Active, ERROR_WRONG_XSTAKE_SC);

        self.xstake_sc().set(address);
    }

    #[storage_mapper("xstake_onedex_id")]
    fn xstake_onedex_id(&self) -> SingleValueMapper<usize>;

    #[only_owner]
    #[endpoint(setXStakeOnedexId)]
    fn set_xstake_onedex_id(&self, id: usize) {
        let onedex_cache = OnedexCache::new(self);
        let onedex_stake = self.get_xstake(id);
        require!(
            onedex_stake.state == xstake_proxy::State::Active &&
            onedex_stake.stake_tokens.len() == 1 &&
            onedex_stake.stake_tokens.get(0).clone_value() == onedex_cache.lp_info.lp_token,
            ERROR_INVALID_XSTAKE
        );

        self.xstake_onedex_id().set(id);
    }

    #[storage_mapper("xstake_xexchange_id")]
    fn xstake_xexchange_id(&self) -> SingleValueMapper<usize>;

    #[only_owner]
    #[endpoint(setXStakeXexchangeId)]
    fn set_xstake_xexchange_id(&self, id: usize) {
        let xexchange_cache = XexchangeCache::new(self);
        let xexchange_stake = self.get_xstake(id);
        require!(
            xexchange_stake.state == xstake_proxy::State::Active &&
            xexchange_stake.stake_tokens.len() == 1 &&
            xexchange_stake.stake_tokens.get(0).clone_value() == xexchange_cache.lp_info.lp_token,
            ERROR_INVALID_XSTAKE
        );

        self.xstake_xexchange_id().set(id);
    }

    fn add_xstake(&self, stake_id: usize, amount: BigUint, token: TokenIdentifier) {
        if !self.is_xstake_active() {
            return
        }

        let payment =
            EsdtTokenPayment::new(token, 0, amount);
        self.xstake_proxy_obj()
            .contract(self.xstake_sc().get())
            .user_stake(stake_id)
            .with_esdt_transfer(payment)
            .execute_on_dest_context::<()>();
    }

    fn remove_xstake(&self, stake_id: usize, amount: BigUint, token: TokenIdentifier) {
        if !self.is_xstake_active() {
            return
        }

        let user_stake = self.get_user_xstake(stake_id, self.blockchain().get_sc_address());
        let staked = user_stake.staked.get(0).clone_value();
        if staked == 0 {
            return
        }

        let mut amount_to_remove = amount.clone();
        if staked < amount {
            amount_to_remove = staked;
        }
        let mut payments: ManagedVec<EsdtTokenPayment> = ManagedVec::new();
        let payment =
            EsdtTokenPayment::new(token, 0, amount_to_remove);
        payments.push(payment);
        self.xstake_proxy_obj()
            .contract(self.xstake_sc().get())
            .user_unstake(stake_id, payments)
            .execute_on_dest_context::<()>();
    }

    fn claim_xstake_rewards(&self, stake_id: usize) {
        if !self.is_xstake_active() {
            return
        }

        self.xstake_proxy_obj()
            .contract(self.xstake_sc().get())
            .claim_rewards(stake_id)
            .execute_on_dest_context::<()>();
    }

    fn get_xstake(&self, stake_id: usize) -> Stake<Self::Api> {
        self.xstake_proxy_obj()
            .contract(self.xstake_sc().get())
            .get_stake(stake_id)
            .execute_on_dest_context()
    }

    fn get_user_xstake(&self, stake_id: usize, user: ManagedAddress) -> UserStake<Self::Api> {
        self.xstake_proxy_obj()
            .contract(self.xstake_sc().get())
            .get_user_stake(stake_id, user)
            .execute_on_dest_context()
    }

    fn take_xstake_profit(&self, stake_id: usize, storage_cache: &mut StorageCache<Self>) {
        if !self.is_xstake_active() {
            return
        }

        let user_stake = self.get_user_xstake(stake_id, self.blockchain().get_sc_address());
        let mut has_rewards = false;
        for reward in user_stake.rewards.iter() {
            if reward.clone_value() > 0 {
                has_rewards = true;
                break
            }
        }
        if has_rewards {
            self.claim_xstake_rewards(stake_id);
        }
        let stake = self.get_xstake(stake_id);
        let wegld_id = self.wegld_id().get();
        let (old_egld_balance, _) = self.get_sc_balances();
        for token in stake.reward_tokens.iter() {
            let balance = self.blockchain()
                .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(token.clone_value()), 0);
            if balance == 0 {
                continue
            }

            let mut path: MultiValueEncoded<TokenIdentifier> = MultiValueEncoded::new();
            path.push(token.clone_value());
            path.push(wegld_id.clone());
            let payment =
                EsdtTokenPayment::new(token.clone_value(), 0, balance.clone());
            self.onedex_proxy_obj()
                .contract(self.onedex_sc().get())
                .swap_multi_tokens_fixed_input(1_u64, true, path)
                .with_esdt_transfer(payment)
                .execute_on_dest_context::<()>();
        }
        let (new_egld_balance, _) = self.get_sc_balances();
        let profit = new_egld_balance - old_egld_balance;
        let egld_profit = &profit / 2_u64;
        let legld_profit = &profit - &egld_profit;
        self.excess_lp_egld().update(|value| *value += egld_profit);

        if legld_profit > 0 {
            let ls_amount =
                self.add_liquidity(&legld_profit, true, storage_cache);
            self.mint_liquid_token(ls_amount.clone());
            storage_cache.egld_to_delegate += legld_profit;
            self.excess_lp_legld().update(|value| *value += ls_amount);
        }
    }

    // proxies

    #[proxy]
    fn xstake_proxy_obj(&self) -> xstake_proxy::Proxy<Self::Api>;
}

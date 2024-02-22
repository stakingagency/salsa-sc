multiversx_sc::imports!();

use crate::common::config::State;
use crate::common::errors::*;
use crate::proxies::xstake_proxy::{self};

#[multiversx_sc::module]
pub trait XStakeModule:
{
    #[only_owner]
    #[endpoint(setXStakeActive)]
    fn set_xstake_active(&self) {
        require!(
            !self.xstake_sc().is_empty(),
            ERROR_XSTAKE_SC,
        );

        self.xstake_state().set(State::Active);
    }

    #[only_owner]
    #[endpoint(setXStakeInactive)]
    fn set_xstake_inactive(&self) {
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
        self.xstake_sc().set(address);
    }

    fn add_xstake(&self, stake_id: usize, amount: BigUint, token: TokenIdentifier) {
        let payment =
            EsdtTokenPayment::new(token, 0, amount);
        self.xstake_proxy_obj()
            .contract(self.xstake_sc().get())
            .user_stake(stake_id)
            .with_esdt_transfer(payment)
            .execute_on_dest_context::<()>();
    }

    fn remove_xstake(&self, stake_id: usize, amount: BigUint, token: TokenIdentifier) {
        let mut payments: ManagedVec<EsdtTokenPayment> = ManagedVec::new();
        let payment =
            EsdtTokenPayment::new(token, 0, amount);
        payments.push(payment);
        self.xstake_proxy_obj()
            .contract(self.xstake_sc().get())
            .user_unstake(stake_id, payments)
            .execute_on_dest_context::<()>();
    }

    fn claim_xstake_rewards(&self, stake_id: usize) {
        self.xstake_proxy_obj()
            .contract(self.xstake_sc().get())
            .claim_rewards(stake_id)
            .execute_on_dest_context::<()>();
    }

    // proxies

    #[proxy]
    fn xstake_proxy_obj(&self) -> xstake_proxy::Proxy<Self::Api>;
}

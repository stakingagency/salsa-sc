multiversx_sc::imports!();

use crate::{common::errors::*};

#[multiversx_sc::module]
pub trait LiquidityModule:
    crate::common::config::ConfigModule
{
    fn add_liquidity(&self, new_stake_amount: &BigUint, update_storage: bool) -> BigUint {
        let total_egld_staked = self.total_egld_staked().get();
        let liquid_token_supply = self.liquid_token_supply().get();
        let ls_amount = if total_egld_staked > 0 {
            if liquid_token_supply == 0 {
                new_stake_amount + &total_egld_staked
            } else {
                new_stake_amount * &liquid_token_supply / &total_egld_staked
            }
        } else {
            new_stake_amount.clone()
        };

        require!(ls_amount > 0, ERROR_NOT_ENOUGH_LIQUID_SUPPLY);

        if update_storage {
            self.total_egld_staked()
                .update(|value| *value += new_stake_amount);
            self.liquid_token_supply()
               .update(|value| *value += &ls_amount);
        }

        ls_amount
    }

    fn remove_liquidity(&self, ls_amount: &BigUint, update_storage: bool) -> BigUint {
        let total_egld_staked = self.total_egld_staked().get();
        let liquid_token_supply = self.liquid_token_supply().get();
        require!(
            &liquid_token_supply >= ls_amount,
            ERROR_NOT_ENOUGH_LIQUID_SUPPLY
        );
        require!(ls_amount > &0, ERROR_BAD_PAYMENT_AMOUNT);

        let egld_amount = ls_amount * &total_egld_staked / &liquid_token_supply;
        require!(egld_amount > 0u64, ERROR_BAD_PAYMENT_AMOUNT);

        if update_storage {
            self.total_egld_staked()
                .update(|value| *value -= &egld_amount);
            self.liquid_token_supply()
                .update(|value| *value -= ls_amount);
        }

        egld_amount
    }

    fn mint_liquid_token(&self, amount: BigUint) -> EsdtTokenPayment<Self::Api> {
        self.liquid_token_id().mint(amount)
    }

    fn burn_liquid_token(&self, amount: &BigUint) {
        self.liquid_token_id().burn(amount);
    }
}
multiversx_sc::imports!();

use crate::common::config::Undelegation;
use crate::{common::errors::*};

#[multiversx_sc::module]
pub trait HelpersModule:
    crate::common::config::ConfigModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    fn add_undelegation(
        &self,
        amount: BigUint,
        unbond_epoch: u64,
        mut list: LinkedListMapper<Undelegation<Self::Api>>
    ) {
        let new_undelegation = Undelegation {
            amount: amount.clone(),
            unbond_epoch,
        };
        let mut found = false;
        for node in list.iter() {
            let node_id = node.get_node_id();
            let mut undelegation = node.into_value();
            if unbond_epoch < undelegation.unbond_epoch {
                list.push_before_node_id(node_id, new_undelegation.clone());
                found = true;
                break
            }
            if unbond_epoch == undelegation.unbond_epoch {
                undelegation.amount += amount;
                list.set_node_value_by_id(node_id, undelegation);
                found = true;
                break
            }
        }
        if !found {
            list.push_back(new_undelegation);
        }

        // merge
        let current_epoch = self.blockchain().get_block_epoch();
        let mut amount_to_merge = BigUint::zero();
        loop {
            let first = match list.front() {
                Some(value) => value,
                None => {
                    break
                }
            };
            let node_id = first.get_node_id();
            let undelegation = first.clone().into_value();
            if current_epoch >= undelegation.unbond_epoch {
                amount_to_merge += undelegation.amount;
                list.remove_node_by_id(node_id);
            } else {
                break
            }
        }
        if amount_to_merge > 0 {
            list.push_front(Undelegation {
                amount: amount_to_merge,
                unbond_epoch: current_epoch
            });
        }
    }

    fn remove_undelegations(
        &self,
        amount: BigUint,
        ref_epoch: u64,
        list: LinkedListMapper<Undelegation<Self::Api>>,
        mut clone_list: LinkedListMapper<Undelegation<Self::Api>>
    ) -> (BigUint, u64) { // left amount, last epoch
        let mut total_amount = amount;
        let mut last_epoch = 0u64;
        for node in list.iter() {
            let mut modified = false;
            let node_id = node.get_node_id();
            let mut undelegation = node.clone().into_value();
            if undelegation.unbond_epoch <= ref_epoch && total_amount > 0 {
                if total_amount >= undelegation.amount {
                    total_amount -= undelegation.amount;
                    undelegation.amount = BigUint::zero();
                } else {
                    undelegation.amount -= total_amount;
                    total_amount = BigUint::zero();
                    last_epoch = undelegation.unbond_epoch;
                    modified = true;
                }
            }
            if undelegation.amount == 0 {
                clone_list.remove_node_by_id(node_id.clone());
            }
            if modified {
                clone_list.set_node_value_by_id(node_id, undelegation);
            }
        }

        (total_amount, last_epoch)
    }

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
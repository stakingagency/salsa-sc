multiversx_sc::imports!();

use crate::common::config::{Undelegation, UndelegationType};
use crate::{common::errors::*};

#[multiversx_sc::module]
pub trait HelpersModule:
    crate::common::config::ConfigModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    fn get_sc_balances(&self) -> (BigUint, BigUint) {
        let liquid_token_id = self.liquid_token_id().get_token_id();
        let ls_balance = self.blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(liquid_token_id.clone()), 0);
        let balance = self.blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0);

        (balance, ls_balance)
    }

    fn get_buy_quantity(&self, egld_amount: BigUint, ls_amount: BigUint, egld_reserve: BigUint, ls_reserve: BigUint) -> BigUint {
        require!(ls_amount > 0, ERROR_INSUFFICIENT_AMOUNT);

        let mut x = &egld_reserve * &ls_reserve * &egld_amount / &ls_amount;
        x = x.sqrt();
        if x < egld_reserve {
            return BigUint::zero()
        }

        x -= egld_reserve;
        if x > egld_amount {
            egld_amount
        } else {
            x
        }
    }

    fn get_sell_quantity(&self, ls_amount: BigUint, egld_amount: BigUint, ls_reserve: BigUint, egld_reserve: BigUint) -> BigUint {
        require!(egld_amount > 0, ERROR_INSUFFICIENT_AMOUNT);
        require!(ls_amount > 0, ERROR_INSUFFICIENT_AMOUNT);

        let mut x = &egld_reserve * &ls_reserve * &egld_amount / &ls_amount;
        x = x.sqrt();
        let y = &ls_reserve * &egld_amount / &ls_amount;
        if x < y {
            return BigUint::zero()
        }

        x -= y;
        x = x * &ls_amount / &egld_amount;
        if x > ls_amount {
            ls_amount
        } else {
            x
        }
    }

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
        list_type: UndelegationType,
        user: ManagedAddress
    ) -> (BigUint, u64) { // left amount, last epoch
        let mut clone_list = self.get_undelegations_list(list_type, &user);
        let mut total_amount = amount;
        let current_epoch = self.blockchain().get_block_epoch();
        let unbond_period = self.unbond_period().get();
        let mut last_epoch = &current_epoch + &unbond_period;
        for node in list.iter() {
            let mut modified = false;
            let node_id = node.get_node_id();
            let mut undelegation = node.clone().into_value();
            if undelegation.unbond_epoch <= ref_epoch && total_amount > 0 {
                last_epoch = undelegation.unbond_epoch;
                if total_amount > undelegation.amount {
                    total_amount -= undelegation.amount;
                    undelegation.amount = BigUint::zero();
                } else {
                    undelegation.amount -= total_amount;
                    total_amount = BigUint::zero();
                    modified = true;
                }
            }
            if undelegation.amount == 0 {
                clone_list.remove_node_by_id(node_id.clone());
            } else if modified {
                clone_list.set_node_value_by_id(node_id, undelegation);
            }
        }

        (total_amount, last_epoch)
    }

    fn get_undelegations_list(
        &self,
        list_type: UndelegationType,
        user: &ManagedAddress
    ) -> LinkedListMapper<Undelegation<Self::Api>> {
        if list_type == UndelegationType::UserList {
            self.luser_undelegations(user)
        } else if list_type == UndelegationType::TotalUsersList {
            self.ltotal_user_undelegations()
        } else {
            self.lreserve_undelegations()
        }
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

        if update_storage {
            require!(ls_amount > 0, ERROR_NOT_ENOUGH_LIQUID_SUPPLY);

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

        let egld_amount = ls_amount * &total_egld_staked / &liquid_token_supply;

        if update_storage {
            require!(ls_amount > &0 && egld_amount > 0, ERROR_BAD_PAYMENT_AMOUNT);

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
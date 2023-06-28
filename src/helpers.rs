multiversx_sc::imports!();

use crate::common::config::{Undelegation, UndelegationType};
use crate::common::consts::MIN_EGLD;
use crate::common::storage_cache::StorageCache;
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

    fn get_optimal_quantity(
        &self,
        in_amount: BigUint,
        out_amount: BigUint,
        reserve1: &BigUint,
        reserve2: &BigUint,
        is_buy: bool
    ) -> BigUint {
        if in_amount == 0 || out_amount == 0 {
            return BigUint::zero()
        }

        let (in_reserve, out_reserve) = if is_buy {
            (reserve1, reserve2)
        } else {
            (reserve2, reserve1)
        };
        let mut x = in_reserve * out_reserve * &in_amount / &out_amount;
        x = x.sqrt();
        if &x < in_reserve {
            return BigUint::zero()
        }

        x -= in_reserve;
        if x > in_amount {
            x = in_amount.clone();
        }

        self.adjust_quantity_if_dust_remaining(in_amount, x)
    }

    fn adjust_quantity_if_dust_remaining(&self, in_amount: BigUint, quantity: BigUint) -> BigUint {
        let rest = &in_amount - &quantity;
        if rest < MIN_EGLD && rest > 0 && in_amount >= MIN_EGLD {
            in_amount - MIN_EGLD
        } else {
            quantity
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

    fn get_salsa_amount_out(&self, amount_in: &BigUint, is_buy: bool, storage_cache: &mut StorageCache<Self>) -> BigUint {
        if is_buy {
            self.add_liquidity(amount_in, false, storage_cache)
        } else {
            self.remove_liquidity(amount_in, false, storage_cache)
        }
    }

    fn add_liquidity(&self, new_stake_amount: &BigUint, update_storage: bool, storage_cache: &mut StorageCache<Self>) -> BigUint {
        let mut ls_amount = new_stake_amount + &storage_cache.total_stake;
        if storage_cache.total_stake > 0 && storage_cache.liquid_supply > 0 {
            ls_amount = new_stake_amount * &storage_cache.liquid_supply / &storage_cache.total_stake;
        }

        if update_storage {
            require!(ls_amount > 0, ERROR_NOT_ENOUGH_LIQUID_SUPPLY);

            storage_cache.total_stake += new_stake_amount;
            storage_cache.liquid_supply += &ls_amount;
        }

        ls_amount
    }

    fn remove_liquidity(&self, ls_amount: &BigUint, update_storage: bool, storage_cache: &mut StorageCache<Self>) -> BigUint {
        require!(
            &storage_cache.liquid_supply >= ls_amount,
            ERROR_NOT_ENOUGH_LIQUID_SUPPLY
        );

        let egld_amount = ls_amount * &storage_cache.total_stake / &storage_cache.liquid_supply;

        if update_storage {
            require!(ls_amount > &0 && egld_amount > 0, ERROR_BAD_PAYMENT_AMOUNT);

            storage_cache.total_stake -= &egld_amount;
            storage_cache.liquid_supply -= ls_amount;
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

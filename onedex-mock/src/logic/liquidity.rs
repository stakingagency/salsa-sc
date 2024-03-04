multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::state::State;
use crate::constants::MINIMUM_LIQUIDITY;
use crate::proxy;

#[multiversx_sc::module]
pub trait LiquidityLogicModule:
    crate::storage::common_storage::CommonStorageModule
    + crate::storage::pair_storage::PairStorageModule
    + crate::logic::common::CommonLogicModule
    + crate::logic::amm::AmmLogicModule
{
    /**
     * Add initial liquidity
     *  Pair owner could add initail liquidity
     */
    #[payable("*")]
    #[endpoint(addInitialLiquidity)]
    fn add_initial_liquidity(&self) {
        let [first_payment, second_payment] = self.call_value().multi_esdt();

        require!(
            first_payment.token_identifier != second_payment.token_identifier,
            "error during parse input tokens"
        );

        require!(
            first_payment.amount > BigUint::zero() && second_payment.amount > BigUint::zero(),
            "invalid add_liquidity_payment amount"
        );
        
        let pair_key = (first_payment.token_identifier.clone(), second_payment.token_identifier.clone());
        
        let pair_ids = self.pair_ids();
        require!(
            pair_ids.contains_key(&pair_key),
            "given pair of token identifiers not exist"
        );
        
        let pair_id =  pair_ids.get(&pair_key).unwrap();

        self.require_valid_pair_id(pair_id);
        self.require_pair_owner_or_admin(pair_id);

        // pair should be empty
        require!(
            self.pair_first_token_reserve(pair_id).get() == BigUint::zero(),
            "first_token_reserve must be zero"
        );
        require!(
            self.pair_second_token_reserve(pair_id).get() == BigUint::zero(),
            "second_token_reserve must be zero"
        );

        let first_token_optimal_amount = &first_payment.amount;
        let second_token_optimal_amount = &second_payment.amount;

        // check if min_reserve is satisfied
        self.check_new_reserves_and_set_value(pair_id, first_token_optimal_amount, second_token_optimal_amount);

        self.pair_state(pair_id).set(State::ActiveButNoSwap);

        let k_constant = self.calculate_k_constant(&first_payment.amount, &second_payment.amount);

        require!(
            k_constant > BigUint::zero(),
            "invalid k constant for adding initial liquidity"
        );

        let mut initial_total_liquidity = core::cmp::min(first_payment.amount.clone(), second_payment.amount.clone());
        let minimum_liquidity = BigUint::from(MINIMUM_LIQUIDITY);

        require!(
            initial_total_liquidity > minimum_liquidity,
            "Initial total liquidity must be greater than minimun liquidity"
        );
        
        let one18 = BigUint::from(10u64).pow(18);
        let ten = BigUint::from(10u64);
        while initial_total_liquidity < one18 {
            initial_total_liquidity *= &ten;
        }

        // mint LP token and send it to caller
        let lp_token_id = self.pair_lp_token_id(pair_id).get();
        self.send().esdt_local_mint(&lp_token_id, 0, &initial_total_liquidity);
        
        // lock minimum liquidity permanently
        // returen the rest liquidity to initial liquidity provider
        let initial_liquidity = &initial_total_liquidity - &minimum_liquidity;

        self.send().direct_esdt(
            &self.blockchain().get_caller(),
            &lp_token_id,
            0,
            &initial_liquidity,
        );
        
        self.pair_lp_token_supply(pair_id).set(initial_total_liquidity.clone());
            }


    /**
     * Add Liquidity
     *  anyone could add liquidity
     */
    #[payable("*")]
    #[endpoint(addLiquidity)]
    fn add_liquidity(
        &self,
        first_token_amount_min: BigUint,
        second_token_amount_min: BigUint,
    ) {
        require!(
            first_token_amount_min > 0 && second_token_amount_min > 0,
            "first and second token min amount must be greater than 0"
        );

        let [first_payment, second_payment] = self.call_value().multi_esdt();

        require!(
            first_payment.token_identifier != second_payment.token_identifier,
            "error during parse input tokens"
        );

        require!(
            first_payment.amount > BigUint::zero() && second_payment.amount > BigUint::zero(),
            "invalid add_liquidity payment amount"
        );

        // check if pair exists and get pair_id
        let pair_key = (first_payment.token_identifier.clone(), second_payment.token_identifier.clone());

        let pair_ids = self.pair_ids();
        require!(
            pair_ids.contains_key(&pair_key),
            "given pair of token identifiers does not exist"
        );
        
        let pair_id =  pair_ids.get(&pair_key).unwrap();

        // check status
        self.require_pair_active(pair_id);
        self.require_pair_is_ready(pair_id);

        // get current token pair reserve
        let old_first_token_reserve = self.pair_first_token_reserve(pair_id).get();
        let old_second_token_reserve = self.pair_second_token_reserve(pair_id).get();

        let second_token_amount_optimal = self.quote(&first_payment.amount, &old_first_token_reserve, &old_second_token_reserve);

        let (first_token_amount_added, second_token_amount_added) = if second_token_amount_optimal <= second_payment.amount {
            (first_payment.amount.clone(), second_token_amount_optimal)
        } else {
            let first_token_amount_optimal = self.quote(&second_payment.amount, &old_second_token_reserve, &old_first_token_reserve);
            require!(
                first_token_amount_optimal <= first_payment.amount,
                "should be first_token_amount_optimal <= first_payment.amount"
            );

            (first_token_amount_optimal, second_payment.amount.clone())
        };
        require!(
            first_token_amount_added >= first_token_amount_min,
            "insufficient first token amount"
        );

        require!(
            second_token_amount_added >= second_token_amount_min,
            "insufficient second token amount"
        );

        // return left tokens
        if first_token_amount_added < first_payment.amount {
            self.send().direct_esdt(
                &self.blockchain().get_caller(),
                &first_payment.token_identifier,
                0,
                &(first_payment.amount.clone() - &first_token_amount_added)
            );
        }
        if second_token_amount_added < second_payment.amount {
            self.send().direct_esdt(
                &self.blockchain().get_caller(),
                &second_payment.token_identifier,
                0,
                &(second_payment.amount.clone() - &second_token_amount_added)
            );
        }

        // calculate old k
        let old_k = self.calculate_k_constant(
            &old_first_token_reserve,
            &old_second_token_reserve,
        );

        // calculate new k
        let new_first_token_reserve = old_first_token_reserve.clone() + &first_token_amount_added;
        let new_second_token_reserve = old_second_token_reserve.clone() + &second_token_amount_added;

        let new_k = self.calculate_k_constant(
            &new_first_token_reserve,
            &new_second_token_reserve
        );

        // check if new k is valid
        require!(
            old_k <= new_k,
            "k invariant failed"
        );

        // set new reserve
        self.pair_first_token_reserve(pair_id).set(&new_first_token_reserve);
        self.pair_second_token_reserve(pair_id).set(&new_second_token_reserve);

        // calculate new lp
        let old_lp_token_supply = self.pair_lp_token_supply(pair_id).get();

        let first_potential_lp = first_token_amount_added * &old_lp_token_supply / &old_first_token_reserve;
        let second_potential_lp = second_token_amount_added * &old_lp_token_supply / &old_second_token_reserve;
        
        let new_liquidity_added = core::cmp::min(first_potential_lp, second_potential_lp);

        require!(new_liquidity_added > BigUint::zero(), "insufficient liquidity minted");

        let new_lp_token_supply = old_lp_token_supply + &new_liquidity_added;

        // mint new lp
        let lp_token_id = self.pair_lp_token_id(pair_id).get();
        self.send().esdt_local_mint(&lp_token_id, 0, &new_liquidity_added);

        // update total supply
        self.pair_lp_token_supply(pair_id).set(new_lp_token_supply);

        // send minted lp token to liquidity provider
        self.send().direct_esdt(
            &self.blockchain().get_caller(),
            &lp_token_id,
            0,
            &new_liquidity_added,
        );
    }


    /**
     * Remove liquidity
     */
    #[payable("*")]
    #[endpoint(removeLiquidity)]
    fn remove_liquidity(
        &self,
        first_token_amount_min: BigUint,
        second_token_amount_min: BigUint,
        unwrap_required: bool
    ) {
        let lp_payment = self.call_value().single_esdt();

        require!(
            self.lp_token_pair_id_map().contains_key(&lp_payment.token_identifier),
            "lp_token_id not exist"
        );

        let pair_id = self.lp_token_pair_id_map().get(&lp_payment.token_identifier).unwrap();

        require!(
            lp_payment.token_identifier == self.pair_lp_token_id(pair_id).get(),
            "lp_token_id not match"
        );

        // check pair status
        self.require_pair_active(pair_id);
        self.require_pair_is_ready(pair_id);

        // get current lp token status
        let old_first_token_reserve = self.pair_first_token_reserve(pair_id).get();
        let old_second_token_reserve = self.pair_second_token_reserve(pair_id).get();

        let old_lp_token_supply = self.pair_lp_token_supply(pair_id).get();

        require!(
            lp_payment.amount <= old_lp_token_supply,
            "cannot withdraw more than lp_token_supply"
        );

        let new_lp_token_supply = old_lp_token_supply.clone() - &lp_payment.amount;

        self.pair_lp_token_supply(pair_id).set(&new_lp_token_supply);

        // burn LP token
        self.send().esdt_local_burn(&lp_payment.token_identifier, 0, &lp_payment.amount);

        // withdraw amount
        let first_token_withdraw_amount = old_first_token_reserve.clone() * &lp_payment.amount / &old_lp_token_supply;
        let second_token_withdraw_amount = old_second_token_reserve.clone() * &lp_payment.amount / &old_lp_token_supply;

        require!(
            first_token_withdraw_amount >= first_token_amount_min,
            "insufficient first_token_amount"
        );
        require!(
            second_token_withdraw_amount >= second_token_amount_min,
            "insufficient second_token_amount"
        );

        let wegld_token = self.wegld_id().get();
        if self.pair_first_token_id(pair_id).get() == wegld_token && unwrap_required {
            let mut unwrap_payment = ManagedVec::new();
            unwrap_payment.push(
                EsdtTokenPayment::new(
                    wegld_token.clone(),
                    0,
                    first_token_withdraw_amount.clone()
                )
            );

            self.unwrap_proxy(
                self.unwrap_address().get()
            )
                .unwrap_egld()
                .with_multi_token_transfer(unwrap_payment)
                .execute_on_dest_context::<()>();
            
            self.send().direct_egld(
                &self.blockchain().get_caller(),
                &first_token_withdraw_amount.clone()
            );
        } else {
            self.send().direct_esdt(
                &self.blockchain().get_caller(),
                &self.pair_first_token_id(pair_id).get(),
                0,
                &first_token_withdraw_amount
            );
        }

        if self.pair_second_token_id(pair_id).get() == wegld_token && unwrap_required {
            let mut unwrap_payment = ManagedVec::new();
            unwrap_payment.push(
                EsdtTokenPayment::new(
                    wegld_token,
                    0,
                    second_token_withdraw_amount.clone()
                )
            );
            
            self.unwrap_proxy(
                self.unwrap_address().get()
            )
                .unwrap_egld()
                .with_multi_token_transfer(unwrap_payment)
                .execute_on_dest_context::<()>();
            
            self.send().direct_egld(
                &self.blockchain().get_caller(),
                &second_token_withdraw_amount.clone()
            );
        } else {
            self.send().direct_esdt(
                &self.blockchain().get_caller(),
                &self.pair_second_token_id(pair_id).get(),
                0,
                &second_token_withdraw_amount
            );
        }


        // set new token reserve
        let new_first_token_reserve = old_first_token_reserve.clone() - &first_token_withdraw_amount;
        let new_second_token_reserve = old_second_token_reserve.clone() - &second_token_withdraw_amount;

        self.check_new_reserves_and_set_value(pair_id, &new_first_token_reserve, &new_second_token_reserve);
    }

    #[inline]
    fn check_new_reserves_and_set_value(
        &self,
        pair_id: usize,
        new_first_token_reserve: &BigUint,
        new_second_token_reserve: &BigUint,
    ) {
        self.pair_first_token_reserve(pair_id).set(new_first_token_reserve);
        self.pair_second_token_reserve(pair_id).set(new_second_token_reserve);
    }

    #[proxy]
    fn unwrap_proxy(&self, sc_address: ManagedAddress) -> proxy::Proxy<Self::Api>;
}
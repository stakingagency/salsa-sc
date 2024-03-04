multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::constants::TOTAL_PERCENT;


#[multiversx_sc::module]
pub trait AmmLogicModule:
    crate::storage::common_storage::CommonStorageModule
    + crate::storage::pair_storage::PairStorageModule
{
    /**
     * k = x * y
     *  x: first token reserve
     *  y: second token reserve
     */
    fn calculate_k_constant(
        &self,
        first_token_amount: &BigUint,
        second_token_amount: &BigUint,
    ) -> BigUint {
        first_token_amount * second_token_amount
    }
    
    /**
     * Calculate Optimal Amount
     */
    fn quote(
        &self,
        first_token_amount: &BigUint,
        first_token_reserve: &BigUint,
        second_token_reserve: &BigUint,
    ) -> BigUint {
        &(first_token_amount * second_token_reserve) / first_token_reserve
    }

    /**
     * Calculate output amount based on input amount (no fee)
     */
    fn get_amount_out_no_fee(
        &self,
        amount_in: &BigUint,
        reserve_in: &BigUint,
        reserve_out: &BigUint,
    ) -> BigUint {
        let numerator = amount_in * reserve_out;
        let denominator = reserve_in + amount_in;

        numerator / denominator
    }

    /**
     * Calculate input amount based on output amount (no fee)
     */
    fn get_amount_in_no_fee(
        &self,
        amount_out: &BigUint,
        reserve_in: &BigUint,
        reserve_out: &BigUint,
    ) -> BigUint {
        let numerator = reserve_in * amount_out;
        let denominator = reserve_out - amount_out;

        (numerator / denominator) + &BigUint::from(1u64)
    }

    fn get_amount_out(
        &self,
        amount_in: &BigUint,
        reserve_in: &BigUint,
        reserve_out: &BigUint,
        fee_input_token: bool
    ) -> BigUint {
        if fee_input_token {
            let amount_in_with_fee = amount_in * (TOTAL_PERCENT - self.total_fee_percent().get());
            let numerator = &amount_in_with_fee * reserve_out;
            let denominator = (reserve_in * TOTAL_PERCENT) + amount_in_with_fee;
    
            numerator / denominator
        } else {
            let amount_out_without_fee = self.get_amount_out_no_fee(amount_in, reserve_in, reserve_out);

            amount_out_without_fee * (TOTAL_PERCENT - self.total_fee_percent().get()) / TOTAL_PERCENT
        }
    }

    fn get_amount_in(
        &self,
        amount_out: &BigUint,
        reserve_in: &BigUint,
        reserve_out: &BigUint,
        fee_input_token: bool
    ) -> BigUint {
        if fee_input_token {
            let numerator = reserve_in * amount_out * TOTAL_PERCENT;
            let denominator =
                (reserve_out - amount_out) * (TOTAL_PERCENT - self.total_fee_percent().get());
    
            (numerator / denominator) + 1u64
        } else {
            let amount_out_with_fee = amount_out * TOTAL_PERCENT / (TOTAL_PERCENT - self.total_fee_percent().get());
            
            self.get_amount_in_no_fee(&amount_out_with_fee, reserve_in, reserve_out)
        }
    }
}

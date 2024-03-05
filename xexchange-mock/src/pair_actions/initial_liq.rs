use crate::{
    config::State, contexts::{add_liquidity::AddLiquidityContext, base::StorageCache}, ERROR_ACTIVE, ERROR_BAD_PAYMENT_TOKENS, ERROR_INITIAL_LIQUIDITY_ALREADY_ADDED, ERROR_PERMISSION_DENIED
};

use super::common_result_types::AddLiquidityResultType;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait InitialLiquidityModule:
    crate::liquidity_pool::LiquidityPoolModule
    + crate::amm::AmmModule
    + crate::contexts::output_builder::OutputBuilderModule
    + crate::config::ConfigModule
    + super::common_methods::CommonMethodsModule
{
    #[payable("*")]
    #[endpoint(addInitialLiquidity)]
    fn add_initial_liquidity(&self) -> AddLiquidityResultType<Self::Api> {
        let mut storage_cache = StorageCache::new(self);
        let caller = self.blockchain().get_caller();

        let opt_initial_liq_adder = self.initial_liquidity_adder().get();
        if let Some(initial_liq_adder) = opt_initial_liq_adder {
            require!(caller == initial_liq_adder, ERROR_PERMISSION_DENIED);
        }

        let [first_payment, second_payment] = self.call_value().multi_esdt();
        require!(
            first_payment.token_identifier == storage_cache.first_token_id
                && first_payment.amount > 0,
            ERROR_BAD_PAYMENT_TOKENS
        );
        require!(
            second_payment.token_identifier == storage_cache.second_token_id
                && second_payment.amount > 0,
            ERROR_BAD_PAYMENT_TOKENS
        );
        require!(
            !self.is_state_active(storage_cache.contract_state),
            ERROR_ACTIVE
        );
        require!(
            storage_cache.lp_token_supply == 0,
            ERROR_INITIAL_LIQUIDITY_ALREADY_ADDED
        );

        let first_token_optimal_amount = &first_payment.amount;
        let second_token_optimal_amount = &second_payment.amount;
        let liq_added = self.pool_add_initial_liquidity(
            first_token_optimal_amount,
            second_token_optimal_amount,
            &mut storage_cache,
        );

        self.send()
            .esdt_local_mint(&storage_cache.lp_token_id, 0, &liq_added);

        let lp_payment =
            EsdtTokenPayment::new(storage_cache.lp_token_id.clone(), 0, liq_added.clone());

        self.send()
            .direct_non_zero_esdt_payment(&caller, &lp_payment);

        self.state().set(State::PartialActive);

        let add_liq_context = AddLiquidityContext {
            first_payment: first_payment.clone(),
            second_payment: second_payment.clone(),
            first_token_amount_min: BigUint::from(1u32),
            second_token_amount_min: BigUint::from(1u32),
            first_token_optimal_amount: first_token_optimal_amount.clone(),
            second_token_optimal_amount: second_token_optimal_amount.clone(),
            liq_added,
        };
        self.build_add_initial_liq_results(&storage_cache, &add_liq_context)
    }
}

use crate::{
    contexts::{add_liquidity::AddLiquidityContext, base::StorageCache}, ERROR_BAD_PAYMENT_TOKENS,
    ERROR_INITIAL_LIQUIDITY_NOT_ADDED, ERROR_INVALID_ARGS, ERROR_K_INVARIANT_FAILED,
    ERROR_LP_TOKEN_NOT_ISSUED, ERROR_NOT_ACTIVE,
};

multiversx_sc::imports!();

use super::common_result_types::AddLiquidityResultType;

#[multiversx_sc::module]
pub trait AddLiquidityModule:
    crate::liquidity_pool::LiquidityPoolModule
    + crate::amm::AmmModule
    + crate::contexts::output_builder::OutputBuilderModule
    + crate::config::ConfigModule
    + super::common_methods::CommonMethodsModule
    + super::token_send::TokenSendModule
{
    #[payable("*")]
    #[endpoint(addLiquidity)]
    fn add_liquidity(
        &self,
        first_token_amount_min: BigUint,
        second_token_amount_min: BigUint,
    ) -> AddLiquidityResultType<Self::Api> {
        require!(
            first_token_amount_min > 0 && second_token_amount_min > 0,
            ERROR_INVALID_ARGS
        );

        let mut storage_cache = StorageCache::new(self);
        let caller = self.blockchain().get_caller();

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
            self.is_state_active(storage_cache.contract_state),
            ERROR_NOT_ACTIVE
        );
        require!(
            storage_cache.lp_token_id.is_valid_esdt_identifier(),
            ERROR_LP_TOKEN_NOT_ISSUED
        );
        require!(
            self.initial_liquidity_adder().get().is_none() || storage_cache.lp_token_supply != 0,
            ERROR_INITIAL_LIQUIDITY_NOT_ADDED
        );

        let initial_k = self.calculate_k_constant(
            &storage_cache.first_token_reserve,
            &storage_cache.second_token_reserve,
        );

        let mut add_liq_context = AddLiquidityContext::new(
            first_payment,
            second_payment,
            first_token_amount_min,
            second_token_amount_min,
        );
        self.set_optimal_amounts(&mut add_liq_context, &storage_cache);

        add_liq_context.liq_added = if storage_cache.lp_token_supply == 0u64 {
            self.pool_add_initial_liquidity(
                &add_liq_context.first_token_optimal_amount,
                &add_liq_context.second_token_optimal_amount,
                &mut storage_cache,
            )
        } else {
            self.pool_add_liquidity(
                &add_liq_context.first_token_optimal_amount,
                &add_liq_context.second_token_optimal_amount,
                &mut storage_cache,
            )
        };

        let new_k = self.calculate_k_constant(
            &storage_cache.first_token_reserve,
            &storage_cache.second_token_reserve,
        );
        require!(initial_k <= new_k, ERROR_K_INVARIANT_FAILED);

        self.send()
            .esdt_local_mint(&storage_cache.lp_token_id, 0, &add_liq_context.liq_added);

        let lp_payment = EsdtTokenPayment::new(
            storage_cache.lp_token_id.clone(),
            0,
            add_liq_context.liq_added.clone(),
        );

        let mut output_payments =
            self.build_add_liq_output_payments(&storage_cache, &add_liq_context);
        output_payments.push(lp_payment);

        self.send_multiple_tokens_if_not_zero(&caller, &output_payments);

        self.build_add_liq_results(&storage_cache, &add_liq_context)
    }
}

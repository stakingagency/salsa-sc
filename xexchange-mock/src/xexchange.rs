#![no_std]

use crate::{config::State, errors::*, consts::*};

multiversx_sc::imports!();

pub mod errors;
pub mod consts;
pub mod config;
pub mod fee;
mod contexts;
pub mod pair_actions;
mod amm;
mod liquidity_pool;

#[multiversx_sc::contract]
pub trait XexchangeMock<ContractReader>:
    amm::AmmModule
    + liquidity_pool::LiquidityPoolModule
    + config::ConfigModule
    + contexts::output_builder::OutputBuilderModule
    + pair_actions::initial_liq::InitialLiquidityModule
    + pair_actions::add_liq::AddLiquidityModule
    + pair_actions::remove_liq::RemoveLiquidityModule
    + pair_actions::swap::SwapModule
    + pair_actions::views::ViewsModule
    + pair_actions::common_methods::CommonMethodsModule
    + pair_actions::token_send::TokenSendModule
    + crate::fee::FeeModule
{
    #[init]
    fn init(
        &self,
        first_token_id: TokenIdentifier,
        second_token_id: TokenIdentifier,
    ) {
        require!(first_token_id.is_valid_esdt_identifier(), ERROR_NOT_AN_ESDT);
        require!(
            second_token_id.is_valid_esdt_identifier(),
            ERROR_NOT_AN_ESDT
        );
        require!(first_token_id != second_token_id, ERROR_SAME_TOKENS);

        let lp_token_id = self.lp_token_identifier().get();
        require!(first_token_id != lp_token_id, ERROR_POOL_TOKEN_IS_PLT);
        require!(second_token_id != lp_token_id, ERROR_POOL_TOKEN_IS_PLT);

        self.set_fee_percents(TOTAL_FEE, SPECIAL_FEE);
        self.state().set(State::Inactive);

        self.first_token_id().set_if_empty(&first_token_id);
        self.second_token_id().set_if_empty(&second_token_id);
    }

    #[only_owner]
    #[endpoint(setLpTokenIdentifier)]
    fn set_lp_token_identifier(&self, token_identifier: TokenIdentifier) {
        require!(
            self.lp_token_identifier().is_empty(),
            ERROR_LP_TOKEN_NOT_ISSUED
        );
        require!(
            token_identifier != self.first_token_id().get()
                && token_identifier != self.second_token_id().get(),
            ERROR_LP_TOKEN_SAME_AS_POOL_TOKENS
        );
        require!(
            token_identifier.is_valid_esdt_identifier(),
            ERROR_NOT_AN_ESDT
        );
        self.lp_token_identifier().set(&token_identifier);
    }
}

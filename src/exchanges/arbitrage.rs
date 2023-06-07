multiversx_sc::imports!();

use crate::common::{errors::*, config::State};

#[multiversx_sc::module]
pub trait ArbitrageModule:
    crate::common::config::ConfigModule
    + crate::helpers::HelpersModule
    + crate::exchanges::onedex::OnedexModule
    + crate::exchanges::xexchange::XexchangeModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[only_owner]
    #[endpoint(setArbitrageActive)]
    fn set_arbitrage_active(&self) {
        require!(!self.provider_address().is_empty(), ERROR_PROVIDER_NOT_SET);
        require!(!self.liquid_token_id().is_empty(), ERROR_TOKEN_NOT_SET);
        require!(
            !self.wegld_id().is_empty(),
            ERROR_WEGLD_ID,
        );

        self.arbitrage().set(State::Active);
    }

    #[only_owner]
    #[endpoint(setArbitrageInactive)]
    fn set_arbitrage_inactive(&self) {
        self.arbitrage().set(State::Inactive);
    }

    #[inline]
    fn is_arbitrage_active(&self) -> bool {
        let arbitrage = self.arbitrage().get();
        arbitrage == State::Active
    }

    #[view(getArbitrageState)]
    #[storage_mapper("arbitrage")]
    fn arbitrage(&self) -> SingleValueMapper<State>;

    fn do_arbitrage(
        &self, in_token: &TokenIdentifier, in_amount: &BigUint
    ) -> (BigUint, BigUint) {
        let mut sold_amount = BigUint::zero();
        let mut bought_amount = BigUint::zero();
        if self.is_arbitrage_active() {
            let is_buy = in_token == &self.wegld_id().get();
            let mut out_amount = if is_buy {
                self.add_liquidity(&in_amount, false)
            } else {
                self.remove_liquidity(&in_amount, false)
            };
            let mut new_in_amount = in_amount.clone();
            if self.is_onedex_arbitrage_active() {
                let (sold, bought) =
                    self.do_arbitrage_on_onedex(in_token, in_amount, &out_amount);
                sold_amount += &sold;
                bought_amount += &bought;
                new_in_amount -= sold;
                out_amount = if is_buy {
                    self.add_liquidity(&new_in_amount, false)
                } else {
                    self.remove_liquidity(&new_in_amount, false)
                };
            }
            if self.is_xexchange_arbitrage_active() {
                let (sold, bought) =
                    self.do_arbitrage_on_xexchange(in_token, &new_in_amount, &out_amount);
                sold_amount += sold;
                bought_amount += bought;
            }
        }

        (sold_amount, bought_amount)
    }
}

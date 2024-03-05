multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use super::errors::*;

pub const MAX_PERCENTAGE: u64 = 100_000;
pub const MAX_FEE_PERCENTAGE: u64 = 5_000;

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Copy, Clone, Debug)]
pub enum State {
    Inactive,
    Active,
    PartialActive,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Eq)]
pub struct TokenPair<M: ManagedTypeApi> {
    pub first_token: TokenIdentifier<M>,
    pub second_token: TokenIdentifier<M>,
}

impl<M: ManagedTypeApi> TokenPair<M> {
    pub fn equals(&self, other: &TokenPair<M>) -> bool {
        self.first_token == other.first_token && self.second_token == other.second_token
    }
}

#[multiversx_sc::module]
pub trait ConfigModule {
    fn set_fee_percents(&self, total_fee_percent: u64, special_fee_percent: u64) {
        require!(
            total_fee_percent >= special_fee_percent && total_fee_percent <= MAX_FEE_PERCENTAGE,
            ERROR_BAD_PERCENTS
        );
        self.total_fee_percent().set(total_fee_percent);
        self.special_fee_percent().set(special_fee_percent);
    }

    #[view(getTotalFeePercent)]
    #[storage_mapper("total_fee_percent")]
    fn total_fee_percent(&self) -> SingleValueMapper<u64>;

    #[view(getSpecialFee)]
    #[storage_mapper("special_fee_percent")]
    fn special_fee_percent(&self) -> SingleValueMapper<u64>;

    #[view(getLpTokenIdentifier)]
    fn get_lp_token_identifier(&self) -> TokenIdentifier {
        self.lp_token_identifier().get()
    }

    #[storage_mapper("lpTokenIdentifier")]
    fn lp_token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getFirstTokenId)]
    #[storage_mapper("first_token_id")]
    fn first_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getSecondTokenId)]
    #[storage_mapper("second_token_id")]
    fn second_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getTotalSupply)]
    #[storage_mapper("lp_token_supply")]
    fn lp_token_supply(&self) -> SingleValueMapper<BigUint>;

    #[view(getReserve)]
    #[storage_mapper("reserve")]
    fn pair_reserve(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(getInitialLiquidtyAdder)]
    #[storage_mapper("initial_liquidity_adder")]
    fn initial_liquidity_adder(&self) -> SingleValueMapper<Option<ManagedAddress>>;

    // pausable
    #[view(getState)]
    #[storage_mapper("state")]
    fn state(&self) -> SingleValueMapper<State>;
}

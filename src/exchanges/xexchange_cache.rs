multiversx_sc::imports!();

use crate::{common::config::{LpInfo, Exchange}, proxies::xexchange_proxy::State};

use super::xexchange::XexchangeModule;

pub struct XexchangeCache<'a, X>
where
    X: crate::common::config::ConfigModule,
{
    sc_ref: &'a X,
    pub sc_address: ManagedAddress<X::Api>,
    pub wrap_sc_address: ManagedAddress<X::Api>,
    pub lp_info: LpInfo<X::Api>,
    pub is_active: bool,
    pub fee: u64,
}

impl<'a, X> XexchangeCache<'a, X>
where
    X: XexchangeModule,
{
    pub fn new(sc_ref: &'a X) -> Self {
        let state = sc_ref.get_xexchange_state();
        let is_active = state == State::Active;
        let (first_reserve, second_reserve, lp_supply) =
            sc_ref.get_xexchange_reserves();
        let lp_token = sc_ref.xexchange_lp_token().get();
        let lp_info = LpInfo {
            exchange: Exchange::Xexchange,
            liquid_reserve: first_reserve,
            egld_reserve: second_reserve,
            lp_supply,
            lp_token,
            lp_balance: sc_ref.xexchange_lp_balance().get(),
        };
        let fee = sc_ref.get_xexchange_fee();
            
        XexchangeCache {
            sc_ref,
            sc_address: sc_ref.xexchange_sc().get(),
            wrap_sc_address: sc_ref.wrap_sc().get(),
            lp_info,
            is_active,
            fee,
        }
    }
}

impl<'a, X> Drop for XexchangeCache<'a, X>
where
    X: crate::common::config::ConfigModule,
{
    fn drop(&mut self) {
        self.sc_ref.xexchange_lp_balance().set(&self.lp_info.lp_balance);
    }
}

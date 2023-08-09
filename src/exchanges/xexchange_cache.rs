multiversx_sc::imports!();

use crate::{common::config::{LpInfo, Exchange}, proxies::xexchange_proxy::State};

use super::xexchange::XexchangeModule;

pub struct XexchangeCache<X>
where
    X: XexchangeModule,
{
    pub sc_address: ManagedAddress<X::Api>,
    pub wrap_sc_address: ManagedAddress<X::Api>,
    pub lp_info: LpInfo<X::Api>,
    pub is_active: bool,
}

impl<'a, X> XexchangeCache<X>
where
    X: XexchangeModule,
{
    pub fn new(sc_ref: &'a X) -> Self {
        let state = sc_ref.get_xexchange_state();
        let is_active = state == State::Active;
        let (first_reserve, second_reserve, lp_supply) =
            sc_ref.get_xexchange_reserves();
        let lp_token = sc_ref.xexchange_lp().get();
        let lp_balance = sc_ref.blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(lp_token.clone()), 0);
        let lp_info = LpInfo {
            exchange: Exchange::Xexchange,
            liquid_reserve: first_reserve,
            egld_reserve: second_reserve,
            lp_supply,
            lp_token,
            lp_balance,
        };
            
        XexchangeCache {
            sc_address: sc_ref.xexchange_sc().get(),
            wrap_sc_address: sc_ref.wrap_sc().get(),
            lp_info,
            is_active,
        }
    }
}

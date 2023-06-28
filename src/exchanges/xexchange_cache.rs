multiversx_sc::imports!();

use super::xexchange::XexchangeModule;

pub struct XexchangeCache<X>
where
    X: XexchangeModule,
{
    pub sc_address: ManagedAddress<X::Api>,
    pub wrap_sc_address: ManagedAddress<X::Api>,
    pub liquid_reserve: BigUint<X::Api>,
    pub egld_reserve: BigUint<X::Api>,
    pub lp_supply: BigUint<X::Api>,
    pub lp_token: TokenIdentifier<X::Api>,
}

impl<'a, X> XexchangeCache<X>
where
    X: XexchangeModule,
{
    pub fn new(sc_ref: &'a X) -> Self {
        let (first_reserve, second_reserve, lp_supply) =
            sc_ref.get_xexchange_reserves();
        
        XexchangeCache {
            sc_address: sc_ref.xexchange_sc().get(),
            wrap_sc_address: sc_ref.wrap_sc().get(),
            liquid_reserve: first_reserve,
            egld_reserve: second_reserve,
            lp_supply: lp_supply,
            lp_token: sc_ref.xexchange_lp().get(),
        }
    }
}

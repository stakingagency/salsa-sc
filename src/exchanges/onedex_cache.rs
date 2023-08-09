multiversx_sc::imports!();

use crate::{common::config::{LpInfo, Exchange}, proxies::onedex_proxy::State};

use super::onedex::OnedexModule;

pub struct OnedexCache<O>
where
    O: OnedexModule,
{
    pub pair_id: usize,
    pub sc_address: ManagedAddress<O::Api>,
    pub lp_info: LpInfo<O::Api>,
    pub is_active: bool,
}

impl<'a, O> OnedexCache<O>
where
    O: OnedexModule,
{
    pub fn new(sc_ref: &'a O) -> Self {
        let pair = sc_ref.get_onedex_pair_info();
        let lp_balance = sc_ref.blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(pair.lp_token_id.clone()), 0);
        let lp_info = LpInfo {
            exchange: Exchange::Onedex,
            liquid_reserve: pair.first_token_reserve,
            egld_reserve: pair.second_token_reserve,
            lp_supply: pair.lp_token_supply,
            lp_token: pair.lp_token_id,
            lp_balance,
        };
        let is_active = pair.enabled && (pair.state == State::Active);
        
        OnedexCache {
            pair_id: sc_ref.onedex_pair_id().get(),
            sc_address: sc_ref.onedex_sc().get(),
            lp_info,
            is_active,
        }
    }
}

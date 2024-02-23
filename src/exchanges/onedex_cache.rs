multiversx_sc::imports!();

use crate::{common::config::{LpInfo, Exchange}, proxies::onedex_proxy::State};

use super::onedex::OnedexModule;

pub struct OnedexCache<'a, O>
where
    O: crate::common::config::ConfigModule,
{
    sc_ref: &'a O,
    pub pair_id: usize,
    pub sc_address: ManagedAddress<O::Api>,
    pub lp_info: LpInfo<O::Api>,
    pub is_active: bool,
    pub fee: u64,
}

impl<'a, O> OnedexCache<'a, O>
where
    O: OnedexModule,
{
    pub fn new(sc_ref: &'a O) -> Self {
        let pair = sc_ref.get_onedex_pair_info();
        let lp_info = LpInfo {
            exchange: Exchange::Onedex,
            liquid_reserve: pair.first_token_reserve,
            egld_reserve: pair.second_token_reserve,
            lp_supply: pair.lp_token_supply,
            lp_token: pair.lp_token_id,
            lp_balance: sc_ref.onedex_lp_balance().get(),
        };
        let is_active = pair.enabled && (pair.state == State::Active);
        let fee = sc_ref.get_onedex_fee();
        
        OnedexCache {
            sc_ref,
            pair_id: sc_ref.onedex_pair_id().get(),
            sc_address: sc_ref.onedex_sc().get(),
            lp_info,
            is_active,
            fee,
        }
    }
}

impl<'a, O> Drop for OnedexCache<'a, O>
where
    O: crate::common::config::ConfigModule,
{
    fn drop(&mut self) {
        self.sc_ref.onedex_lp_balance().set(&self.lp_info.lp_balance);
    }
}

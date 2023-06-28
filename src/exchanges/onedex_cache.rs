multiversx_sc::imports!();

use super::onedex::OnedexModule;

pub struct OnedexCache<O>
where
    O: OnedexModule,
{
    pub pair_id: usize,
    pub sc_address: ManagedAddress<O::Api>,
    pub liquid_reserve: BigUint<O::Api>,
    pub egld_reserve: BigUint<O::Api>,
    pub lp_supply: BigUint<O::Api>,
    pub lp_token: TokenIdentifier<O::Api>,
}

impl<'a, O> OnedexCache<O>
where
    O: OnedexModule,
{
    pub fn new(sc_ref: &'a O) -> Self {
        let pair = sc_ref.get_onedex_pair_info();
        
        OnedexCache {
            pair_id: sc_ref.onedex_pair_id().get(),
            sc_address: sc_ref.onedex_sc().get(),
            liquid_reserve: pair.first_token_reserve,
            egld_reserve: pair.second_token_reserve,
            lp_supply: pair.lp_token_supply,
            lp_token: pair.lp_token_id,
        }
    }
}

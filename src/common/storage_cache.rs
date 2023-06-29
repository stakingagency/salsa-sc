multiversx_sc::imports!();

pub struct StorageCache<'a, C>
where
    C: crate::common::config::ConfigModule,
{
    sc_ref: &'a C,
    pub total_stake: BigUint<C::Api>,
    pub liquid_supply: BigUint<C::Api>,
    pub liquid_token_id: TokenIdentifier<C::Api>,
    pub wegld_id: TokenIdentifier<C::Api>,
    pub legld_in_custody: BigUint<C::Api>,
    pub available_egld_reserve: BigUint<C::Api>,
}

impl<'a, C> StorageCache<'a, C>
where
    C: crate::common::config::ConfigModule,
{
    pub fn new(sc_ref: &'a C) -> Self {
        StorageCache {
            sc_ref,
            total_stake: sc_ref.total_egld_staked().get(),
            liquid_supply: sc_ref.liquid_token_supply().get(),
            liquid_token_id: sc_ref.liquid_token_id().get_token_id(),
            wegld_id: sc_ref.wegld_id().get(),
            legld_in_custody: sc_ref.legld_in_custody().get(),
            available_egld_reserve: sc_ref.available_egld_reserve().get(),
        }
    }
}

impl<'a, C> Drop for StorageCache<'a, C>
where
    C: crate::common::config::ConfigModule,
{
    fn drop(&mut self) {
        self.sc_ref.total_egld_staked().set(&self.total_stake);
        self.sc_ref.liquid_token_supply().set(&self.liquid_supply);
        self.sc_ref.legld_in_custody().set(&self.legld_in_custody);
        self.sc_ref.available_egld_reserve().set(&self.available_egld_reserve);
    }
}

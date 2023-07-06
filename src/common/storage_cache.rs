multiversx_sc::imports!();

pub struct StorageCache<'a, C>
where
    C: crate::common::config::ConfigModule,
{
    sc_ref: &'a C,
    pub unbond_period: u64,
    pub total_stake: BigUint<C::Api>,
    pub liquid_supply: BigUint<C::Api>,
    pub liquid_token_id: TokenIdentifier<C::Api>,
    pub wegld_id: TokenIdentifier<C::Api>,
    pub legld_in_custody: BigUint<C::Api>,
    pub available_egld_reserve: BigUint<C::Api>,
    pub egld_to_undelegate: BigUint<C::Api>,
    pub egld_reserve: BigUint<C::Api>,
    pub reserve_points: BigUint<C::Api>,
}

impl<'a, C> StorageCache<'a, C>
where
    C: crate::common::config::ConfigModule,
{
    pub fn new(sc_ref: &'a C) -> Self {
        StorageCache {
            sc_ref,
            unbond_period: sc_ref.unbond_period().get(),
            total_stake: sc_ref.total_egld_staked().get(),
            liquid_supply: sc_ref.liquid_token_supply().get(),
            liquid_token_id: sc_ref.liquid_token_id().get_token_id(),
            wegld_id: sc_ref.wegld_id().get(),
            legld_in_custody: sc_ref.legld_in_custody().get(),
            available_egld_reserve: sc_ref.available_egld_reserve().get(),
            egld_to_undelegate: sc_ref.egld_to_undelegate().get(),
            egld_reserve: sc_ref.egld_reserve().get(),
            reserve_points: sc_ref.reserve_points().get(),
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
        self.sc_ref.egld_to_undelegate().set(&self.egld_to_undelegate);
        self.sc_ref.egld_reserve().set(&self.egld_reserve);
        self.sc_ref.reserve_points().set(&self.reserve_points);
    }
}

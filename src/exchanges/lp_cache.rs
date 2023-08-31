multiversx_sc::imports!();

pub struct LpCache<'a, L>
where
    L: crate::common::config::ConfigModule,
{
    sc_ref: &'a L,
    pub egld_in_lp: BigUint<L::Api>,
    pub legld_in_lp: BigUint<L::Api>,
    pub excess_lp_egld: BigUint<L::Api>,
    pub excess_lp_legld: BigUint<L::Api>,
}

impl<'a, L> LpCache<'a, L>
where
    L: crate::common::config::ConfigModule,
{
    pub fn new(sc_ref: &'a L) -> Self {
        LpCache {
            sc_ref,
            egld_in_lp: sc_ref.egld_in_lp().get(),
            legld_in_lp: sc_ref.legld_in_lp().get(),
            excess_lp_egld: sc_ref.excess_lp_egld().get(),
            excess_lp_legld: sc_ref.excess_lp_legld().get(),
        }
    }
}

impl<'a, L> Drop for LpCache<'a, L>
where
    L: crate::common::config::ConfigModule,
{
    fn drop(&mut self) {
        self.sc_ref.egld_in_lp().set(&self.egld_in_lp);
        self.sc_ref.legld_in_lp().set(&self.legld_in_lp);
        self.sc_ref.excess_lp_egld().set(&self.excess_lp_egld);
        self.sc_ref.excess_lp_legld().set(&self.excess_lp_legld);
    }
}

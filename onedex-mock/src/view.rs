multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::state::Pair;
use crate::constants::LP_TOKEN_DECIMALS;

#[multiversx_sc::module]
pub trait ViewModule:
    crate::storage::common_storage::CommonStorageModule
    + crate::storage::pair_storage::PairStorageModule
{
    #[view(viewPair)]
    fn view_pair(&self, pair_id: usize) -> Pair<Self::Api> {
        let roles = self.blockchain().get_esdt_local_roles(&self.pair_lp_token_id(pair_id).get());
        let lp_token_roles_are_set = roles.has_role(&EsdtLocalRole::Mint) && roles.has_role(&EsdtLocalRole::Burn);

        Pair {
            pair_id,
            state: self.pair_state(pair_id).get(),
            owner: self.pair_owner(pair_id).get(),
            enabled: self.pair_enabled(pair_id).get(),

            first_token_id: self.pair_first_token_id(pair_id).get(),
            second_token_id: self.pair_second_token_id(pair_id).get(),
            lp_token_id: self.pair_lp_token_id(pair_id).get(),

            lp_token_decimal: LP_TOKEN_DECIMALS,

            first_token_reserve: self.pair_first_token_reserve(pair_id).get(),
            second_token_reserve: self.pair_second_token_reserve(pair_id).get(),
            lp_token_supply: self.pair_lp_token_supply(pair_id).get(),

            lp_token_roles_are_set
        }
    }
}

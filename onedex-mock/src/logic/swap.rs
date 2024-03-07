multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::constants::TOTAL_PERCENT;
use crate::proxy;

#[multiversx_sc::module]
pub trait SwapLogicModule:
    crate::storage::common_storage::CommonStorageModule
    + crate::storage::pair_storage::PairStorageModule
    + crate::logic::common::CommonLogicModule
    + crate::logic::amm::AmmLogicModule
    + crate::logic::liquidity::LiquidityLogicModule
{
    /**
     * SWAP Fixed Input
     */
    #[payable("*")]
    #[endpoint(swapMultiTokensFixedInput)]
    fn swap_multi_tokens_fixed_input(
        &self,
        amount_out_min: BigUint,
        unwrap_required: bool,
        path_args: MultiValueEncoded<TokenIdentifier>,
    ) {
        let (token_in, amount_in_arg) = self.call_value().egld_or_single_fungible_esdt();

        require!(
            amount_in_arg > BigUint::zero(),
            "Must be paid some amount to swap"
        );

        let path = path_args.to_vec();

        let path_len = path.len();
        require!(
            path_len >= 2,
            "Length of path must be equal to or bigger than 2"
        );

        let wegld_token_id = self.wegld_id().get();

        if token_in.is_egld() {
            require!(
                wegld_token_id == *path.get(0),
                "payment token and first path token does not match"
            );

            self.proxy_sc(
                self.unwrap_address().get()
            )
                .wrap_egld()
                .with_egld_transfer(amount_in_arg.clone())
                .execute_on_dest_context::<()>();
        } else {
            require!(
                token_in.unwrap_esdt() == *path.get(0),
                "payment token and first path token does not match"
            );
        }

        let mut amount_in = amount_in_arg.clone();
        let mut amount_out = BigUint::zero();

        for i in 0..path_len - 1 {
            let token_in = path.get(i);
            let token_out = path.get(i + 1);

            let (output, _, _) = self.perform_swap_fixed_input(&token_in, &token_out, &amount_in);
            amount_out = output.clone();
            amount_in = output.clone();
        }

        require!(
            amount_out >= amount_out_min,
            "Insufficient output token computed amount"
        );

        let token_out: &TokenIdentifier = &path.get(path_len - 1);

        if unwrap_required {
            require!(
                token_out == &self.wegld_id().get(),
                "Only unwrap available for wegld"
            );

            let mut unwrap_payment = ManagedVec::new();
            unwrap_payment.push(
                EsdtTokenPayment::new(
                    self.wegld_id().get(),
                    0,
                    amount_out.clone()
                )
            );

            self.proxy_sc(
                self.unwrap_address().get()
            )
            .unwrap_egld()
            .with_multi_token_transfer(unwrap_payment)
            .execute_on_dest_context::<()>();

            self.send().direct_egld(
                &self.blockchain().get_caller(),
                &amount_out
            );
        } else {
            self.send().direct_esdt(
                &self.blockchain().get_caller(),
                token_out,
                0,
                &amount_out
            );
        }
    }


    /**
     * SWAP Fixed Output
     */
    #[payable("*")]
    #[endpoint(swapMultiTokensFixedOutput)]
    fn swap_multi_tokens_fixed_output(
        &self,
        amount_out_wanted: BigUint,
        unwrap_required: bool,
        path_args: MultiValueEncoded<TokenIdentifier>,
    ) {
        let (token_in, amount_in_arg) = self.call_value().egld_or_single_fungible_esdt();

        require!(
            amount_in_arg > BigUint::zero(),
            "Must be paid some amount to swap"
        );

        let path = path_args.to_vec();

        let path_len = path.len();
        require!(
            path_len >= 2,
            "Length of path must be equal to or bigger than 2"
        );

        let wegld_token_id = self.wegld_id().get();

        if token_in.is_egld() {
            require!(
                wegld_token_id == *path.get(0),
                "payment token and first path token does not match"
            );

            self.proxy_sc(
                self.unwrap_address().get()
            )
                .wrap_egld()
                .with_egld_transfer(amount_in_arg.clone())
                .execute_on_dest_context::<()>();
        } else {
            require!(
                token_in.unwrap_esdt() == *path.get(0),
                "payment token and first path token does not match"
            );
        }

        let mut amount_in = BigUint::zero();
        let mut amount_out = amount_out_wanted.clone();

        for i in 0..path_len - 1 {
            let token_in = path.get(path_len - i - 2);
            let token_out = path.get(path_len - i - 1);

            let (input, _, _) = self.perform_swap_fixed_output(&token_in, &token_out, &amount_out);
            amount_in = input.clone();
            amount_out = input.clone();
        }

        amount_out = amount_out_wanted.clone();

        require!(
            amount_in > BigUint::zero() && amount_in <= amount_in_arg.clone(),
            "Insufficient input token computed amount"
        );

        // refund the redendance fund
        if amount_in < amount_in_arg {
            self.send().direct_esdt(
                &self.blockchain().get_caller(),
                &(*path.get(0)),
                0,
                &(amount_in_arg - &amount_in),
            );
        }

        let token_out: &TokenIdentifier = &path.get(path_len - 1);

        if unwrap_required {
            require!(
                token_out == &self.wegld_id().get(),
                "Only unwrap available for wegld"
            );

            let mut unwrap_payment = ManagedVec::new();
            unwrap_payment.push(
                EsdtTokenPayment::new(
                    self.wegld_id().get(),
                    0,
                    amount_out.clone()
                )
            );

            self.proxy_sc(
                self.unwrap_address().get()
            )
                .unwrap_egld()
                .with_multi_token_transfer(unwrap_payment)
                .execute_on_dest_context::<()>();

            self.send().direct_egld(
                &self.blockchain().get_caller(),
                &amount_out
            );
        } else {
            self.send().direct_esdt(
                &self.blockchain().get_caller(),
                token_out,
                0,
                &amount_out
            );
        }
    }

    /**
     * Swap Fixed Input
     *  return: amount_out
     */
    fn perform_swap_fixed_input(
        &self,
        token_in: &TokenIdentifier,
        token_out: &TokenIdentifier,
        amount_in: &BigUint,
    ) -> (BigUint, BigUint, BigUint) {
        let pair_id = self.get_pair_id(token_in, token_out);

        self.require_pair_active_swap(pair_id);
        self.require_pair_is_ready(pair_id);

        let fee_input_token = self.main_pair_tokens().contains(token_in);

        // get token reserve before swap
        let first_token_id = self.pair_first_token_id(pair_id).get();
        let old_first_token_reserve = self.pair_first_token_reserve(pair_id).get();
        let old_second_token_reserve = self.pair_second_token_reserve(pair_id).get();

        let token_in_is_first_id = true;
        let (
            amount_out,
            new_first_token_reserve,
            new_second_token_reserve
        ) =
            if token_in == &first_token_id {
                self.swap_fixed_input(token_in, token_out, amount_in, &old_first_token_reserve, &old_second_token_reserve, fee_input_token)
            } else if token_out == &first_token_id {
                let (
                    amount_out,
                    new_second_token_reserve,
                    new_first_token_reserve,
                ) = self.swap_fixed_input(token_in, token_out, amount_in, &old_second_token_reserve, &old_first_token_reserve, fee_input_token);

                (
                    amount_out,
                    new_first_token_reserve,
                    new_second_token_reserve,
                )
            } else {
                sc_panic!("Input or output token must be first token of the pair");
            };

        self.check_new_reserves_and_set_value(pair_id, &new_first_token_reserve, &new_second_token_reserve);

        let mut token_in_reserve = &new_first_token_reserve;
        let mut token_out_reserve = &new_second_token_reserve;
        if !token_in_is_first_id {
            token_in_reserve = &new_second_token_reserve;
            token_out_reserve = &new_first_token_reserve;
        }

        (
            amount_out,
            token_in_reserve.clone(),
            token_out_reserve.clone(),
        )
    }

    fn perform_swap_fixed_output(
        &self,
        token_in: &TokenIdentifier,
        token_out: &TokenIdentifier,
        amount_out: &BigUint,
    ) -> (BigUint, BigUint, BigUint) {
        let pair_id = self.get_pair_id(token_in, token_out);

        // check
        self.require_pair_active_swap(pair_id);
        self.require_pair_is_ready(pair_id);

        // get token reserve before swap
        let first_token_id = self.pair_first_token_id(pair_id).get();
        let old_first_token_reserve = self.pair_first_token_reserve(pair_id).get();
        let old_second_token_reserve = self.pair_second_token_reserve(pair_id).get();

        let fee_input_token = self.main_pair_tokens().contains(token_in);

        let token_in_is_first_id = true;
        let (
            amount_in,
            new_first_token_reserve,
            new_second_token_reserve
        ) =
            if token_in == &first_token_id {
                self.swap_fixed_output(token_in, token_out, amount_out, &old_first_token_reserve, &old_second_token_reserve, fee_input_token)
            } else if token_out == &first_token_id {
                let (
                    amount_in,
                    new_second_token_reserve,
                    new_first_token_reserve
                ) = self.swap_fixed_output(token_in, token_out, amount_out, &old_second_token_reserve, &old_first_token_reserve, fee_input_token);

                (
                    amount_in,
                    new_first_token_reserve,
                    new_second_token_reserve,
                )
            } else {
                sc_panic!("Input or output token must be first token of the pair");
            };

        // update new pair token reserves
        self.check_new_reserves_and_set_value(pair_id, &new_first_token_reserve, &new_second_token_reserve);

        let mut token_in_reserve = &new_first_token_reserve;
        let mut token_out_reserve = &new_second_token_reserve;
        if !token_in_is_first_id {
            token_in_reserve = &new_second_token_reserve;
            token_out_reserve = &new_first_token_reserve;
        }

        (
            amount_in,
            token_in_reserve.clone(),
            token_out_reserve.clone(),
        )
    }

    fn swap_fixed_input(
        &self,
        token_in: &TokenIdentifier,
        token_out: &TokenIdentifier,
        amount_in: &BigUint,
        old_reserve_in: &BigUint,
        old_reserve_out: &BigUint,
        fee_in: bool,
    ) -> (BigUint, BigUint, BigUint) {
        let total_fee_percent = self.total_fee_percent().get();

        if fee_in {
            let total_fee_amount = amount_in * &BigUint::from(total_fee_percent) / &BigUint::from(TOTAL_PERCENT);
            let left_amount_in = amount_in - &total_fee_amount;
            let left_fee_amount = self.manage_special_fee(token_in, &total_fee_amount);

            let amount_out = self.get_amount_out_no_fee(&left_amount_in, old_reserve_in, old_reserve_out);
            let new_reserve_in = old_reserve_in + &left_amount_in + &left_fee_amount;
            let new_reserve_out = old_reserve_out - &amount_out;

            (amount_out, new_reserve_in, new_reserve_out)
        } else {
            let amount_out = self.get_amount_out_no_fee(amount_in, old_reserve_in, old_reserve_out);
            let total_fee_amount = amount_out.clone() * &BigUint::from(total_fee_percent) / &BigUint::from(TOTAL_PERCENT);
            let left_amount_out = amount_out.clone() - &total_fee_amount;
            let left_fee_amount = self.manage_special_fee(token_out, &total_fee_amount);

            let new_reserve_in = old_reserve_in + amount_in;
            let new_reserve_out = old_reserve_out - &amount_out + &left_fee_amount;

            (left_amount_out, new_reserve_in, new_reserve_out)
        }
    }

    fn swap_fixed_output(
        &self,
        token_in: &TokenIdentifier,
        token_out: &TokenIdentifier,
        amount_out: &BigUint,
        old_reserve_in: &BigUint,
        old_reserve_out: &BigUint,
        fee_in: bool,
    ) -> (BigUint, BigUint, BigUint) {
        let total_fee_percent = self.total_fee_percent().get();

        if fee_in {
            let amount_in_with_no_fee = self.get_amount_in_no_fee(amount_out, old_reserve_in, old_reserve_out);
            let total_fee_amount = amount_in_with_no_fee.clone() * &BigUint::from(total_fee_percent) / &BigUint::from(TOTAL_PERCENT - total_fee_percent);
            let amount_in = amount_in_with_no_fee.clone() + &total_fee_amount;

            let left_fee_amount = self.manage_special_fee(token_in, &total_fee_amount);

            let new_reserve_in = old_reserve_in + &amount_in_with_no_fee + &left_fee_amount;
            let new_reserve_out = old_reserve_out - amount_out;

            (amount_in, new_reserve_in, new_reserve_out)
        } else {
            let total_fee_amount = amount_out * &BigUint::from(total_fee_percent) / &BigUint::from(TOTAL_PERCENT - total_fee_percent);
            let total_amount_out = amount_out + &total_fee_amount;
            let left_fee_amount = self.manage_special_fee(token_out, &total_fee_amount);

            let amount_in = self.get_amount_in_no_fee(&total_amount_out, old_reserve_in, old_reserve_out);

            let new_reserve_in = old_reserve_in + &amount_in;
            let new_reserve_out = old_reserve_out - &total_amount_out + &left_fee_amount;

            (amount_in, new_reserve_in, new_reserve_out)
        }
    }

    #[inline]
    fn manage_special_fee(
        &self,
        fee_token: &TokenIdentifier,
        fee_amount: &BigUint,
    ) -> BigUint {
        if self.total_fee_percent().get() == 0 {
            BigUint::zero()
        } else {
            let special_fee_amount = fee_amount.clone() * self.special_fee_percent().get() / self.total_fee_percent().get();
            if special_fee_amount != BigUint::zero() {
                self.send().direct_esdt(
                    &self.treasury_address().get(),
                    fee_token,
                    0,
                    &special_fee_amount
                );
            }

            let staking_reward_fee_amount = fee_amount.clone() * self.staking_reward_fee_percent().get() / self.total_fee_percent().get();
            if staking_reward_fee_amount != BigUint::zero() {
                self.send().direct_esdt(
                    &self.staking_reward_address().get(),
                    fee_token,
                    0,
                    &staking_reward_fee_amount
                );
            }

            fee_amount - &special_fee_amount - &staking_reward_fee_amount
        }

    }

    #[view(getEquivalent)]
    fn get_equivalent(
        &self,
        token_in: TokenIdentifier,
        token_out: TokenIdentifier,
        amount_in: BigUint
    ) -> BigUint {
        require!(
            amount_in > 0u64,
            "Zero amount"
        );

        let zero = BigUint::zero();

        let pair_id = self.get_pair_id(&token_in, &token_out);

        let first_token_id = self.pair_first_token_id(pair_id).get();
        let second_token_id = self.pair_second_token_id(pair_id).get();

        let first_token_reserve = self.pair_first_token_reserve(pair_id).get();
        let second_token_reserve = self.pair_second_token_reserve(pair_id).get();

        if first_token_reserve == 0u64 || second_token_reserve == 0u64 {
            return zero;
        }

        if token_in == first_token_id {
            self.quote(&amount_in, &first_token_reserve, &second_token_reserve)
        } else if token_in == second_token_id {
            self.quote(&amount_in, &second_token_reserve, &first_token_reserve)
        } else {
            sc_panic!("Unknown Token");
        }
    }


    #[view(getMultiPathAmountOut)]
    fn get_multi_path_amount_out(
        &self,
        amount_in_arg: BigUint,
        path_args: MultiValueEncoded<TokenIdentifier>,
    ) -> BigUint {
        require!(
            amount_in_arg > 0u64,
            "Zero amount"
        );

        let path = path_args.to_vec();

        let path_len = path.len();
        require!(
            path_len >= 2,
            "Length of path must be equal to or bigger than 2"
        );

        let mut amount_in = amount_in_arg.clone();
        let mut amount_out = BigUint::zero();

        for i in 0..path_len - 1 {
            let token_in = path.get(i);
            let token_out = path.get(i + 1);

            amount_out = self.get_amount_out_view(&token_in, &token_out, amount_in.clone());
            amount_in = amount_out.clone();
        }

        require!(
            amount_out > 0u64,
            "Insufficient output token computed amount"
        );

        amount_out
    }

    #[view(getAmountOut)]
    fn get_amount_out_view(
        &self,
        token_in: &TokenIdentifier,
        token_out: &TokenIdentifier,
        amount_in: BigUint
    ) -> BigUint {
        require!(
            amount_in > BigUint::zero(),
            "Zero amount"
        );

        let pair_id = self.get_pair_id(token_in, token_out);

        self.require_pair_active_swap(pair_id);

        let first_token_id = &self.pair_first_token_id(pair_id).get();
        let second_token_id = &self.pair_second_token_id(pair_id).get();

        let first_token_reserve = self.pair_first_token_reserve(pair_id).get();
        let second_token_reserve = self.pair_second_token_reserve(pair_id).get();

        let fee_input_token = self.main_pair_tokens().contains(token_in);

        if token_in == first_token_id {
            require!(second_token_reserve > 0u64, "Not enough reserve");

            let amount_out =
                self.get_amount_out(&amount_in, &first_token_reserve, &second_token_reserve, fee_input_token);
            require!(second_token_reserve > amount_out, "Not enough reserve");

            amount_out
        } else if token_in == second_token_id {
            require!(first_token_reserve > 0u64, "Not enough reserve");

            let amount_out =
                self.get_amount_out(&amount_in, &second_token_reserve, &first_token_reserve, fee_input_token);
            require!(first_token_reserve > amount_out, "Not enough reserve");

            amount_out
        } else {
            sc_panic!("Unknown Token");
        }
    }


    #[view(getMultiPathAmountIn)]
    fn get_multi_path_amount_in(
        &self,
        amount_out_wanted: BigUint,
        path_args: MultiValueEncoded<TokenIdentifier>,
    ) -> BigUint {
        require!(
            amount_out_wanted > 0u64,
            "Zero amount"
        );

        let path = path_args.to_vec();

        let path_len = path.len();
        require!(
            path_len >= 2,
            "Length of path must be equal to or bigger than 2"
        );

        let mut amount_in = BigUint::zero();
        let mut amount_out = amount_out_wanted.clone();

        for i in 0..path_len - 1 {
            let token_in = path.get(path_len - i - 2);
            let token_out = path.get(path_len - i - 1);

            amount_in = self.get_amount_in_view(&token_in, &token_out, &amount_out);
            amount_out = amount_in.clone();
        }

        require!(
            amount_in > 0u64,
            "Insufficient output token computed amount"
        );

        amount_in
    }


    #[view(getAmountIn)]
    fn get_amount_in_view(
        &self,
        token_in: &TokenIdentifier,
        token_wanted: &TokenIdentifier,
        amount_wanted: &BigUint
    ) -> BigUint {
        require!(
            amount_wanted > &BigUint::zero(),
            "Zero amount"
        );

        let pair_id = self.get_pair_id(token_in, token_wanted);

        self.require_pair_active_swap(pair_id);

        let first_token_id = &self.pair_first_token_id(pair_id).get();
        let second_token_id = &self.pair_second_token_id(pair_id).get();

        let first_token_reserve = self.pair_first_token_reserve(pair_id).get();
        let second_token_reserve = self.pair_second_token_reserve(pair_id).get();

        let fee_input_token = self.main_pair_tokens().contains(token_in);

        if token_wanted == first_token_id {
            require!(
                &first_token_reserve > amount_wanted,
                "Not enough reserve"
            );

            self.get_amount_in(amount_wanted, &second_token_reserve, &first_token_reserve, fee_input_token)
        } else if token_wanted == second_token_id {
            require!(
                &second_token_reserve > amount_wanted,
                "Not enough reserve"
            );

            self.get_amount_in(amount_wanted, &first_token_reserve, &second_token_reserve, fee_input_token)
        } else {
            sc_panic!("Unknown Token");
        }
    }

    #[proxy]
    fn proxy_sc(&self, sc_address: ManagedAddress) -> proxy::Proxy<Self::Api>;
}
// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           93
// Async Callback:                       1
// Promise callbacks:                    6
// Total number of exported functions: 101

#![no_std]
#![allow(internal_features)]
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    salsa
    (
        init => init
        upgrade => upgrade
        delegate => delegate
        unDelegate => undelegate
        withdraw => withdraw
        addToCustody => add_to_custody
        removeFromCustody => remove_from_custody
        addReserve => add_reserve
        removeReserve => remove_reserve
        unDelegateNow => undelegate_now
        unDelegateKnight => undelegate_knight
        unDelegateNowKnight => undelegate_now_knight
        withdrawKnight => withdraw_knight
        removeReserveKnight => remove_reserve_knight
        unDelegateHeir => undelegate_heir
        unDelegateNowHeir => undelegate_now_heir
        withdrawHeir => withdraw_heir
        removeReserveHeir => remove_reserve_heir
        reduceEgldToDelegateUndelegate => call_reduce_egld_to_delegate_undelegate
        registerLiquidToken => register_liquid_token
        getLiquidTokenId => liquid_token_id
        getLiquidTokenSupply => liquid_token_supply
        setStateActive => set_state_active
        setStateInactive => set_state_inactive
        getState => state
        getProviders => providers
        getUnbondPeriod => unbond_period
        setUnbondPeriod => set_unbond_period
        getServiceFee => service_fee
        setServiceFee => set_service_fee
        getUserUndelegations => luser_undelegations
        getTotalEgldStaked => total_egld_staked
        getUserWithdrawnEgld => user_withdrawn_egld
        getTotalWithdrawnEgld => total_withdrawn_egld
        getTotalUserUndelegations => ltotal_user_undelegations
        getEgldReserve => egld_reserve
        getReservePoints => reserve_points
        getAvailableEgldReserve => available_egld_reserve
        getReserveUndelegations => lreserve_undelegations
        getUsersReservePoints => users_reserve_points
        setUndelegateNowFee => set_undelegate_now_fee
        getUndelegateNowFee => undelegate_now_fee
        getReservePointsAmount => get_reserve_points_amount
        getReserveEgldAmount => get_reserve_egld_amount
        getUserReserve => get_user_reserve
        getTokenPrice => token_price
        setWrapSC => set_wrap_sc
        getLegldInCustody => legld_in_custody
        getUserDelegation => user_delegation
        getUserKnight => user_knight
        getKnightUsers => knight_users
        getUserHeir => user_heir
        getHeirUsers => heir_users
        getContractInfo => get_contract_info
        getUserInfo => get_user_info
        delegateAll => delegate_all
        unDelegateAll => undelegate_all
        claimRewards => claim_rewards
        withdrawAll => withdraw_all
        computeWithdrawn => compute_withdrawn
        setArbitrageActive => set_arbitrage_active
        setArbitrageInactive => set_arbitrage_inactive
        getArbitrageState => arbitrage
        triggerArbitrage => trigger_arbitrage
        setOnedexArbitrageActive => set_onedex_arbitrage_active
        setOnedexArbitrageInactive => set_onedex_arbitrage_inactive
        getOnedexArbitrageState => onedex_arbitrage
        setOnedexSC => set_onedex_sc
        setOnedexPairId => set_onedex_pair_id
        setXexchangeArbitrageActive => set_xexchange_arbitrage_active
        setXexchangeArbitrageInactive => set_xexchange_arbitrage_inactive
        getXexchangeArbitrageState => xexchange_arbitrage
        setXexchangeSC => set_xexchange_sc
        setLpActive => set_lp_active
        setLpInactive => set_lp_inactive
        getLpState => lp_state
        takeLpProfit => take_lp_profit
        setXStakeActive => set_xstake_active
        setXStakeInactive => set_xstake_inactive
        getXStakeState => xstake_state
        setXStakeSC => set_xstake_sc
        setKnight => set_knight
        cancelKnight => cancel_knight
        activateKnight => activate_knight
        deactivateKnight => deactivate_knight
        confirmKnight => confirm_knight
        removeKnight => remove_knight
        setHeir => set_heir
        cancelHeir => cancel_heir
        removeHeir => remove_heir
        updateLastAccessed => update_last_accessed
        addProvider => add_provider
        removeProvider => remove_provider
        setProviderState => set_provider_state
        claim_rewards_callback => claim_rewards_callback
        withdraw_all_callback => withdraw_all_callback
        get_contract_config_callback => get_contract_config_callback
        get_total_active_stake_callback => get_total_active_stake_callback
        get_all_nodes_states_callback => get_all_nodes_states_callback
        get_delegator_funds_data_callback => get_delegator_funds_data_callback
    )
}

multiversx_sc_wasm_adapter::async_callback! { salsa }

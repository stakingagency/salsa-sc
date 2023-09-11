// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           85
// Async Callback:                       1
// Total number of exported functions:  87

#![no_std]
#![allow(internal_features)]
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    salsa
    (
        init => init
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
        flashLoanLEGLD => flash_loan_legld
        flashLoanEGLD => flash_loan_egld
        registerLiquidToken => register_liquid_token
        getLiquidTokenId => liquid_token_id
        getLiquidTokenSupply => liquid_token_supply
        setStateActive => set_state_active
        setStateInactive => set_state_inactive
        getState => state
        setProviderAddress => set_provider_address
        getProviderAddress => provider_address
        getUnbondPeriod => unbond_period
        setUnbondPeriod => set_unbond_period
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
        compound => compound
        withdrawAll => withdraw_all
        computeWithdrawn => compute_withdrawn
        setArbitrageActive => set_arbitrage_active
        setArbitrageInactive => set_arbitrage_inactive
        getArbitrageState => arbitrage
        flashLoanArbitrage => flash_loan_arbitrage
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
    )
}

multiversx_sc_wasm_adapter::async_callback! { salsa }

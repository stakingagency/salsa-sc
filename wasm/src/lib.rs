// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           21
// Async Callback:                       1
// Total number of exported functions:  23

#![no_std]
#![feature(alloc_error_handler, lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    salsa
    (
        delegate
        unDelegate
        withdraw
        compound
        updateTotalEgldStaked
        withdrawAll
        addReserve
        removeReserve
        unDelegateNow
        registerLiquidToken
        setStateActive
        setStateInactive
        setProviderAddress
        setUndelegateNowFee
        getState
        getLiquidTokenId
        getProviderAddress
        getLiquidTokenSupply
        getTotalEgldStaked
        getEgldReserve
        getUndelegateNowFee
        callBack
    )
}

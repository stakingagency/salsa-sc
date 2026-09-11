pub const ONE_EGLD: u64 = 1_000_000_000_000_000_000;
pub const MIN_EGLD: u64 = ONE_EGLD;
pub const MIN_GAS_FOR_ASYNC_CALL: u64 = 12_000_000;
pub const MIN_GAS_FOR_CALLBACK: u64 = 12_000_000;
pub const MIN_GAS_FOR_VIEW_CALL: u64 = 1_000_000;
pub const MIN_GAS_FOR_VIEW_CALLBACK: u64 = 12_000_000;
pub const MIN_GAS_FOR_GET_ALL_NODE_STATES_CALL: u64 = 40_000_000;
/// Extra gas the parent must keep beyond `with_gas_limit` + callback when
/// calling `register_promise`. The VM deducts all of this from the caller's
/// gas left, on top of the call gas and the callback gas:
///   CreateAsyncCall api cost                  200_000
///   UseGasForAsyncStep (AsyncCallStep)        100_000
///   code_size * AoTPreparePerByte      ~50_200 * 100 = 5_020_000
///   AsyncCallStep + AsyncCallbackGasLock    4_100_000
/// which is ~9.42M today. 10M keeps a margin for code sizes up to ~56KB;
/// if the wasm grows past that, this constant must grow with it, otherwise
/// `register_promise` fails with `not enough gas` after the check passed.
pub const GAS_OVERHEAD_PER_PROMISE: u64 = 10_000_000;
/// Gas that must remain after reserving a promise so the parent can finish the loop.
pub const GAS_LEFT_AFTER_PROMISE: u64 = 3_000_000;
pub const MAX_PERCENT: u64 = 10_000;
pub const MIN_UNDELEGATE_NOW_FEE: u64 = 3;
pub const MAX_UNBOND_PERIOD: u64 = 20;
pub const DUST_THRESHOLD: u64 = 1_000;
pub const MIN_INHERITANCE_EPOCHS: u64 = 365;
pub const MAX_INHERITANCE_EPOCHS: u64 = 3653;
pub const MAX_KNIGHT_USERS: usize = 10;
pub const MAX_HEIR_USERS: usize = 10;
pub const MIN_BLOCK_BETWEEN_DELEGATIONS: u64 = 10;
pub const METACHAIN_SHARD_ID: u32 = 4_294_967_295;

pub const PROVIDER_UPDATE_SECONDS_DELTA: u64 = 30 * 60; // 30 minutes
pub const NODE_BASE_STAKE: u64 = 2_500;
pub const MAX_SALSA_FEE: u64 = 1000;
pub const MAX_PROVIDER_FEE: u64 = 2000; // 20%
pub const MAX_PROVIDERS: usize = 25;

pub const PROVIDER_CONFIG_FEE_INDEX: usize = 1;
pub const PROVIDER_CONFIG_MAX_CAP_INDEX: usize = 2;
pub const PROVIDER_CONFIG_HAS_CAP_INDEX: usize = 5;
pub const PROVIDER_FUNDS_REWARDS_INDEX: usize = 1;
pub const PROVIDER_FUNDS_UNDELEGATED_INDEX: usize = 2;
pub const PROVIDER_FUNDS_WITHDRAWABLE_INDEX: usize = 3;
pub const MIN_DELEGATE_AMOUNT: u128 = 50_000_000_000_000_000_000;
pub const MIN_DELEGATE_PERCENT: u64 = 500;

pub const CHALLENGE_DURATION: u64 = 5; // 5 epochs
pub const MIN_DURATION_USERS_CAN_UNDELEGATE: u64 = 5; // 5 epochs

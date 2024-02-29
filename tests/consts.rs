pub const ONE_EGLD: u64 = 1_000_000_000_000_000_000;

pub const SALSA_ADDRESS_EXPR: &str = "sc:salsa";
pub const SALSA_PATH_EXPR: &str = "mxsc:output/salsa.mxsc.json";
pub const TOKEN_ID_EXPR: &str = "str:LEGLD-123456";
pub const TOKEN_ID: &str = "LEGLD-123456";
pub const UNBOND_PERIOD: u64 = 10;
pub const SERVICE_FEE: u64 = 1000;
pub const UNDELEGATE_NOW_FEE: u64 = 200;

pub const DELEGATION1_ADDRESS_EXPR: &str = "sc:delegation1";
pub const DELEGATION1_TOTAL_STAKE: u64 = 15_000;
pub const DELEGATION1_NODES_COUNT: u64 = 5;
pub const DELEGATION1_FEE: u64 = 1000;
pub const DELEGATION1_APR: u64 = 700;
pub const DELEGATION2_ADDRESS_EXPR: &str = "sc:delegation2";
pub const DELEGATION2_TOTAL_STAKE: u64 = 30_000;
pub const DELEGATION2_NODES_COUNT: u64 = 10;
pub const DELEGATION2_FEE: u64 = 800;
pub const DELEGATION2_APR: u64 = 750;
pub const DELEGATION_PATH_EXPR: &str = "mxsc:delegation-mock/output/delegation-mock.mxsc.json";

pub const OWNER_ADDRESS_EXPR: &str = "address:owner";
pub const CALLER_ADDRESS_EXPR: &str = "address:caller";
pub const DELEGATOR1_ADDRESS_EXPR: &str = "address:first-delegator";
pub const DELEGATOR2_ADDRESS_EXPR: &str = "address:second-delegator";
pub const RESERVER1_ADDRESS_EXPR: &str = "address:first-reserver";
pub const RESERVER2_ADDRESS_EXPR: &str = "address:second-reserver";
pub const DELEGATOR1_INITIAL_BALANCE_EXPR: &str = "10_000_000_000_000_000_000_000";
pub const DELEGATOR2_INITIAL_BALANCE_EXPR: &str = "100_000_000_000_000_000_000";
pub const RESERVER1_INITIAL_BALANCE_EXPR: &str = "1_000_000_000_000_000_000_000";
pub const RESERVER2_INITIAL_BALANCE_EXPR: &str = "100_000_000_000_000_000_000";

pub const BLOCKS_PER_EPOCH: u64 = 10;

pub const GAS_LIMIT_DELEGATE_ALL: u64 = 300_000_000;
pub const GAS_LIMIT_UNDELEGATE_ALL: u64 = 300_000_000;
pub const GAS_LIMIT_WITHDRAW_ALL: u64 = 300_000_000;
pub const GAS_LIMIT_CLAIM_REWARDS: u64 = 300_000_000;
pub const GAS_LIMIT_REFRESH_PROVIDERS: u64 = 300_000_000;
pub const GAS_LIMIT_REFRESH_PROVIDER: u64 = 100_000_000;

use hex_literal::hex;

pub const MIN_EGLD_TO_DELEGATE: u64 = 1_000_000_000_000_000_000;
pub const MIN_GAS_FOR_ASYNC_CALL: u64 = 12_000_000;
pub const MIN_GAS_FOR_CALLBACK: u64 = 12_000_000;
pub const UNBOND_PERIOD: u64 = 1;
pub const MAX_PERCENT: u64 = 10_000;
pub const MAX_USER_UNDELEGATIONS: usize = 10;
pub const MAX_RESERVE_UNDELEGATIONS: usize = 20;

pub const WEGLD_ID: &[u8] = b"WEGLD-d7c6bb";
pub const WRAPPER_SC: [u8; 32] =
    hex!("00000000000000000500f12dd10c4d2be8264fe339da14b9fad7bdf364ae7ceb");
pub const ONEDEX_SC: [u8; 32] =
    hex!("000000000000000005004c552ea1e9482e6f60ecdbc5e996c7a86d0d6438b009");

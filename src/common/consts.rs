use hex_literal::hex;

pub const MIN_EGLD: u64 = 1_000_000_000_000_000_000;
pub const MIN_GAS_FOR_ASYNC_CALL: u64 = 12_000_000;
pub const MIN_GAS_FOR_CALLBACK: u64 = 12_000_000;
pub const MAX_PERCENT: u64 = 10_000;
pub const MAX_UNBOND_PERIOD: u64 = 20;
pub const DUST_THRESHOLD: u64 = 1_000;
pub const ARBITRAGE_RATIO: u64 = 9_000;
pub const MIN_INHERITANCE_EPOCHS: u64 = 365;

// devnet consts
pub const WEGLD_ID: &[u8] = b"WEGLD-d7c6bb";
pub const ONEDEX_SC: [u8; 32] =
    hex!("000000000000000005004c552ea1e9482e6f60ecdbc5e996c7a86d0d6438b009");

// mainnet consts
// pub const WEGLD_ID: &[u8] = b"WEGLD-bd4d79";
// pub const ONEDEX_SC: [u8; 32] =
//     hex!("0000000000000000050000b4c094947e427d79931a8bad81316b797d238cdb3f");

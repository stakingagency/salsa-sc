pub static ERROR_INSUFFICIENT_AMOUNT: &[u8] = b"Insufficient amount";
pub static ERROR_INSUFFICIENT_GAS: &[u8] = b"Insufficient gas remaining for the callback";
pub static ERROR_NOT_ACTIVE: &[u8] = b"Not active";
pub static ERROR_ACTIVE: &[u8] = b"Active state";
pub static ERROR_BAD_PAYMENT_TOKEN: &[u8] = b"Bad payment token";
pub static ERROR_BAD_PAYMENT_AMOUNT: &[u8] = b"Insufficient undelegated amount";
pub static ERROR_NOTHING_TO_WITHDRAW: &[u8] = b"Nothing to withdraw";
pub static ERROR_NOT_ENOUGH_FUNDS: &[u8] = b"Not enough funds";
pub static ERROR_USER_NOT_PROVIDER: &[u8] = b"The user is not a reserves provider";
pub static ERROR_PROVIDER_ALREADY_SET: &[u8] = b"Provider address already set";
pub static ERROR_PROVIDER_NOT_SET: &[u8] = b"Provider address not set";
pub static ERROR_UNBOND_PERIOD_NOT_SET: &[u8] = b"Unbond period not set";
pub static ERROR_UNBOND_PERIOD_ALREADY_SET: &[u8] = b"Unbond period already set";
pub static ERROR_TOKEN_ALREADY_SET: &[u8] = b"Token already set";
pub static ERROR_TOKEN_NOT_SET: &[u8] = b"Token not set";
pub static ERROR_NOT_ENOUGH_LIQUID_SUPPLY: &[u8] = b"Not enough liquid token supply";
pub static ERROR_INCORRECT_FEE: &[u8] = b"Fee must be less than 100%";
pub static ERROR_DUST_REMAINING: &[u8] = b"Can't leave dust";
pub static ERROR_ARBITRAGE_ISSUE: &[u8] = b"Arbitrage issue";
pub static ERROR_ONEDEX_PAIR_ID: &[u8] = b"OneDex pair ID not set";
pub static ERROR_FEE_ZERO: &[u8] = b"Fee can't be zero";
pub static ERROR_REMOVE_RESERVE_TOO_SOON: &[u8] = b"You can remove reserve only 1 epoch after add";
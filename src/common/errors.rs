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
pub static ERROR_REMOVE_RESERVE_TOO_SOON: &[u8] = b"You can remove reserve only 1 epoch after add";
pub static ERROR_FEE_CHANGED: &[u8] = b"Fee changed and you would receive less";
pub static ERROR_INSUFFICIENT_FUNDS: &[u8] = b"Insufficient funds";
pub static ERROR_USER_NOT_DELEGATOR: &[u8] = b"You are not a custodial delegator";

pub static ERROR_ARBITRAGE_ISSUE: &[u8] = b"Arbitrage issue";
pub static ERROR_ONEDEX_SC: &[u8] = b"OneDex SC address not set";
pub static ERROR_ONEDEX_PAIR_ID: &[u8] = b"OneDex pair ID not set";
pub static ERROR_XEXCHANGE_SC: &[u8] = b"xExchange SC address not set";
pub static ERROR_WRAP_SC: &[u8] = b"Wrap SC address not set";

pub static ERROR_KNIGHT_ALREADY_SET: &[u8] = b"Knight already set";
pub static ERROR_KNIGHT_SET: &[u8] = b"When you set a knight, unDelegateNow and removeFromCustody are disabled";
pub static ERROR_KNIGHT_NOT_SET: &[u8] = b"Knight not set";
pub static ERROR_KNIGHT_NOT_CONFIRMED: &[u8] = b"Knight not confirmed";
pub static ERROR_KNIGHT_ACTIVE: &[u8] = b"Knight is active";
pub static ERROR_KNIGHT_NOT_ACTIVE: &[u8] = b"Knight not active";
pub static ERROR_NOT_KNIGHT_OF_USER: &[u8] = b"You are not a knight of this user";
pub static ERROR_KNIGHT_YOURSELF: &[u8] = b"You can't be your own knight";
pub static ERROR_KNIGHT_NOT_PENDING: &[u8] = b"Knight can only be canceled or confirmed while pending confirmation";

pub static ERROR_NO_HEIR: &[u8] = b"User has no heir";
pub static ERROR_NOT_HEIR_OF_USER: &[u8] = b"You are not the heir of this user";
pub static ERROR_HEIR_NOT_YET_ENTITLED: &[u8] = b"You are not yet entitled for inheritance";
pub static ERROR_LOW_INHERITANCE_EPOCHS: &[u8] = b"Inheritance should be after at least one year";
pub static ERROR_INHERIT_YOURSELF: &[u8] = b"You can't be your own heir";
pub static ERROR_HEIR_SET: &[u8] = b"When you set a heir, you cannot remove all from custody";

use cosmwasm_std::Addr;
use cw_storey::containers::Item;
 
const ADMIN_ID: u8 = 0;
const DONATION_DENOM_ID: u8 = 1;
pub const ADMINS: Item<Vec<Addr>> = Item::new(ADMIN_ID);
pub const DONATION_DENOM: Item<String> = Item::new(DONATION_DENOM_ID);
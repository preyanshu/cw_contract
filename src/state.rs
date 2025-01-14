use cosmwasm_std::Addr;
use cw_storey::containers::Item;
 
const ADMIN_ID: u8 = 0;
pub const ADMINS: Item<Vec<Addr>> = Item::new(ADMIN_ID);
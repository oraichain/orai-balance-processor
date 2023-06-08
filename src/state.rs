use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_controllers::Admin;
use cw_storage_plus::Map;
use oraiswap::asset::AssetInfo;

#[cw_serde]
pub struct BalanceInfo {
    pub label: String, // label of the asset for human reading
    pub balances: Vec<AssetData>,
}

#[cw_serde]
pub struct AssetData {
    pub asset: AssetInfo,
    // lower balance threshold. Should top-up if actual balance lower.
    pub lower_bound: Uint128,
    // upper balance threshold. Will only top-up to this value.
    pub upper_bound: Uint128,
    // asset decimal
    pub decimals: u8,
}

// Admin of the contract. Can update / edit balance info
pub const ADMIN: Admin = Admin::new("admin");

/// List of balances mapping. Key is an Addr type, and Balance info contains the label of the address, and its mapping balances
pub const BALANCE_INFOS: Map<Addr, BalanceInfo> = Map::new("BALANCE_INFOS");

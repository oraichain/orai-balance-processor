use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use oraiswap::asset::{Asset, AssetInfo};

use crate::state::AssetData;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    /// Append / add new balance array element for a given asset info
    AddBalance(AddNewBalanceMappingMsg),
    /// Update an existing balance array element for a given asset info
    UpdateBalance(UpdateBalanceMappingMsg),
    /// Delete a balance mapping meaning removing the asset info in the mapping
    DeleteBalanceMapping(DeleteBalanceMappingMsg),
    /// Update new admin
    UpdateAdmin { new_admin: String },
}

/// The message to add balance mapping.
///
/// # Propeties
///
/// * `lower_bound` - The lower bound of the balance mapping not multiple decimals
///
#[cw_serde]
pub struct AddNewBalanceMappingMsg {
    pub addr: String,
    pub balance_info: AssetInfo,
    pub lower_bound: Uint128,
    pub decimals: u8,
    pub label: Option<String>,
}

/// The message to update exist balance mapping.
///
/// # Properties
///
/// * `lower_bound` - The lower bound of the balance mapping not multiple decimals
///

#[cw_serde]
pub struct UpdateBalanceMappingMsg {
    pub addr: String,
    pub balance_info: AssetInfo,
    pub lower_bound: Option<Uint128>,
    pub decimals: Option<u8>,
}

#[cw_serde]
pub struct DeleteBalanceMappingMsg {
    pub addr: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Return low balances in the list of balance mapping
    #[returns(QueryLowBalancesResponse)]
    QueryLowBalances {},
    /// Query all list of balance mappings and their current balances
    #[returns(QueryBalancesMappingResponse)]
    QueryBalancesMapping {},
    /// Query a balance mapping given an asset info
    #[returns(QueryBalanceMappingResponse)]
    QueryBalanceMapping { addr: String },
    #[returns(cw_controllers::AdminResponse)]
    QueryAdmin {},
}

#[cw_serde]
pub struct QueryLowBalancesResponse {
    pub low_balance_assets: Vec<BalancesQuery>,
}

#[cw_serde]
pub struct QueryBalancesReponse {
    pub balance_assets: Vec<BalancesMappingQuery>,
}

#[cw_serde]
pub struct QueryBalancesMappingResponse {
    pub balance_assets: Vec<BalancesMappingQuery>,
}

#[cw_serde]
pub struct QueryBalanceMappingResponse {
    pub label: String,
    pub assets: Vec<AssetData>,
}

#[cw_serde]
pub struct BalancesMappingQuery {
    pub addr: Addr,
    pub label: String,
    pub assets: Vec<AssetData>,
}

#[cw_serde]
pub struct BalancesQuery {
    pub addr: Addr,
    pub label: String,
    pub assets: Vec<Asset>,
}

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw_controllers::AdminResponse;
use oraiswap::asset::{Asset, AssetInfo};

use crate::state::AssetData;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    /// Append / add new balance array element for a given asset info
    AddBalance(AddNewBalanceMsg),
    /// Update an existing balance array element for a given asset info
    UpdateBalance(UpdateBalanceMsg),
    /// Delete a balance array element
    DeleteBalance(DeleteBalanceMsg),
    /// Delete a balance mapping meaning removing the asset info in the mapping
    DeleteBalanceMapping(DeleteBalanceMappingMsg),
    /// Topup low balances if needed
    Topup(TopupMsg),
    /// Update new admin
    UpdateAdmin { new_admin: String },
}

#[cw_serde]
pub struct AddNewBalanceMsg {
    pub addr: String,
    pub balance_info: AssetInfo,
    pub lower_bound: Uint128,
    pub upper_bound: Uint128,
    pub label: Option<String>,
}

#[cw_serde]
pub struct UpdateBalanceMsg {
    pub addr: String,
    pub balance_info: AssetInfo,
    pub lower_bound: Uint128,
    pub upper_bound: Uint128,
    pub label: Option<String>,
}

#[cw_serde]
pub struct DeleteBalanceMsg {
    pub addr: String,
    pub balance_info: AssetInfo,
}

#[cw_serde]
pub struct DeleteBalanceMappingMsg {
    pub addr: String,
}

#[cw_serde]
pub struct TopupMsg {
    pub assets: Vec<Asset>,
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
    #[returns(AdminResponse)]
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

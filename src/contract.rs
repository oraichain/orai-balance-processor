#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use oraiswap::asset::AssetInfo;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    AddNewBalanceMsg, DeleteBalanceMappingMsg, DeleteBalanceMsg, ExecuteMsg, InstantiateMsg,
    QueryBalanceMappingResponse, QueryBalancesMappingResponse, QueryLowBalancesResponse, QueryMsg,
    TopupMsg, UpdateBalanceMsg,
};
use crate::state::ADMIN;

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:orai-balance-processor";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    ADMIN.set(deps.branch(), Some(info.sender.clone()))?;
    Ok(Response::new().add_attribute("admin", info.sender.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateAdmin { new_admin } => {
            let new_admin = Some(deps.api.addr_validate(&new_admin)?);
            ADMIN
                .execute_update_admin(deps.branch(), info, new_admin)
                .map_err(|err| ContractError::Std(StdError::generic_err(err.to_string())))
        }
        ExecuteMsg::AddBalance(msg) => add_balance(deps, info, msg),
        ExecuteMsg::UpdateBalance(msg) => update_balance(deps, info, msg),
        ExecuteMsg::DeleteBalance(msg) => delete_balance(deps, info, msg),
        ExecuteMsg::DeleteBalanceMapping(msg) => update_balance_mapping(deps, info, msg),
        ExecuteMsg::Topup(msg) => topup(deps, info, msg),
    }
}

pub fn add_balance(
    deps: DepsMut,
    info: MessageInfo,
    msg: AddNewBalanceMsg,
) -> Result<Response, ContractError> {
    Err(ContractError::Std(StdError::generic_err("unimplemented")))
}

pub fn update_balance(
    deps: DepsMut,
    info: MessageInfo,
    msg: UpdateBalanceMsg,
) -> Result<Response, ContractError> {
    Err(ContractError::Std(StdError::generic_err("unimplemented")))
}

pub fn delete_balance(
    deps: DepsMut,
    info: MessageInfo,
    msg: DeleteBalanceMsg,
) -> Result<Response, ContractError> {
    Err(ContractError::Std(StdError::generic_err("unimplemented")))
}

pub fn update_balance_mapping(
    deps: DepsMut,
    info: MessageInfo,
    msg: DeleteBalanceMappingMsg,
) -> Result<Response, ContractError> {
    Err(ContractError::Std(StdError::generic_err("unimplemented")))
}

pub fn topup(deps: DepsMut, info: MessageInfo, msg: TopupMsg) -> Result<Response, ContractError> {
    Err(ContractError::Std(StdError::generic_err("unimplemented")))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryAdmin {} => to_binary(&ADMIN.query_admin(deps)?),
        QueryMsg::QueryBalanceMapping { asset_info } => {
            to_binary(&query_balance_mapping(deps, asset_info)?)
        }
        QueryMsg::QueryBalancesMapping {} => to_binary(&query_balances_mapping(deps)?),
        QueryMsg::QueryLowBalances {} => to_binary(&query_low_balances(deps)?),
    }
}

pub fn query_balance_mapping(
    deps: Deps,
    asset_info: AssetInfo,
) -> StdResult<QueryBalanceMappingResponse> {
    Err(StdError::generic_err("unimplemented"))
}

pub fn query_balances_mapping(deps: Deps) -> StdResult<QueryBalancesMappingResponse> {
    Err(StdError::generic_err("unimplemented"))
}

pub fn query_low_balances(deps: Deps) -> StdResult<QueryLowBalancesResponse> {
    Err(StdError::generic_err("unimplemented"))
}

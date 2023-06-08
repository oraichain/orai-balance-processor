use std::ops::Mul;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128,
};
use oraiswap::asset::Asset;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::helpers::query_balance;
use crate::msg::{
    AddNewBalanceMappingMsg, BalancesMappingQuery, BalancesQuery, DeleteBalanceMappingMsg,
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryBalanceMappingResponse,
    QueryBalancesMappingResponse, QueryLowBalancesResponse, QueryMsg, UpdateBalanceMappingMsg,
};
use crate::state::{AssetData, BalanceInfo, ADMIN, BALANCE_INFOS};

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
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateAdmin { new_admin } => {
            let new_admin = deps.api.addr_validate(&new_admin).ok();
            Ok(ADMIN.execute_update_admin(deps, info, new_admin)?)
        }
        ExecuteMsg::AddBalance(msg) => add_balance(deps, info, msg),
        ExecuteMsg::UpdateBalance(msg) => update_balance(deps, info, msg),
        ExecuteMsg::DeleteBalanceMapping(msg) => delete_balance_mapping(deps, info, msg),
    }
}

pub fn add_balance(
    deps: DepsMut,
    info: MessageInfo,
    msg: AddNewBalanceMappingMsg,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    let addr = deps.api.addr_validate(&msg.addr)?;

    // if already exist we append new balance into the list
    BALANCE_INFOS.update(
        deps.storage,
        addr,
        |balance_info| -> Result<BalanceInfo, ContractError> {
            let mut balance_info = balance_info.unwrap_or_else(|| BalanceInfo {
                label: msg.label.unwrap_or_default(),
                balances: vec![], // default empty vector
            });

            // if not exist then we append the new balance into the list
            // we dont allow repetitive balance info in the list to prevent spamming
            if balance_info
                .balances
                .iter()
                .any(|asset_data| asset_data.asset.eq(&msg.balance_info))
            {
                return Err(ContractError::BalanceInfoExists {});
            }

            balance_info.balances.push(AssetData {
                asset: msg.balance_info.clone(),
                lower_bound: msg.lower_bound,
                decimals: msg.decimals,
            });

            Ok(balance_info)
        },
    )?;
    // send response
    let res = Response::new().add_attributes(vec![
        attr("action", "add_balance"),
        attr("addr", msg.addr),
        attr("balance_info", msg.balance_info.to_string()),
        attr("lower_bound", msg.lower_bound),
    ]);
    Ok(res)
}

pub fn update_balance(
    deps: DepsMut,
    info: MessageInfo,
    msg: UpdateBalanceMappingMsg,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    let addr = deps.api.addr_validate(&msg.addr)?;

    // if already exist we find the element & update its content
    BALANCE_INFOS.update(
        deps.storage,
        addr,
        |balance_info| -> Result<BalanceInfo, ContractError> {
            let mut balance_info = balance_info.ok_or(ContractError::BalanceMappingNotExist {})?;
            let mut asset_data = balance_info
                .balances
                .iter_mut()
                .find(|a| a.asset.eq(&msg.balance_info))
                .ok_or(ContractError::BalanceInfoNotExist {})?;

            if msg.lower_bound.is_none() {
                return Err(ContractError::Std(StdError::generic_err(
                    "lower_bound and upper_bound not set",
                )));
            }
            asset_data.lower_bound = msg.lower_bound.unwrap_or(asset_data.lower_bound);
            asset_data.decimals = msg.decimals.unwrap_or(asset_data.decimals);

            Ok(balance_info)
        },
    )?;
    // send response
    let res = Response::new().add_attributes(vec![
        attr("action", "update_balance"),
        attr("addr", msg.addr),
        attr("asset_info", msg.balance_info.to_string()),
    ]);
    Ok(res)
}

pub fn delete_balance_mapping(
    deps: DepsMut,
    info: MessageInfo,
    msg: DeleteBalanceMappingMsg,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    BALANCE_INFOS.remove(deps.storage, deps.api.addr_validate(&msg.addr)?);
    let res = Response::new().add_attributes(vec![
        attr("action", "delete_balance_mapping"),
        attr("addr", msg.addr),
    ]);
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryAdmin {} => to_binary(&ADMIN.query_admin(deps)?),
        QueryMsg::QueryBalanceMapping { addr } => to_binary(&query_balance_mapping(deps, addr)?),
        QueryMsg::QueryBalancesMapping {} => to_binary(&query_balances_mapping(deps)?),
        QueryMsg::QueryLowBalances {} => to_binary(&query_low_balances(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

pub fn query_balance_mapping(deps: Deps, addr: String) -> StdResult<QueryBalanceMappingResponse> {
    let balance_query = BALANCE_INFOS.load(deps.storage, deps.api.addr_validate(&addr)?)?;
    Ok(QueryBalanceMappingResponse {
        label: balance_query.label,
        assets: balance_query.balances,
    })
}

pub fn query_balances_mapping(deps: Deps) -> StdResult<QueryBalancesMappingResponse> {
    let infos: Vec<BalancesMappingQuery> = BALANCE_INFOS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| {
            let (k, v) = item?;
            Ok(BalancesMappingQuery {
                addr: k,
                label: v.label,
                assets: v.balances,
            })
        })
        .collect::<StdResult<_>>()?;

    Ok(QueryBalancesMappingResponse {
        balance_assets: infos,
    })
}

pub fn query_low_balances(deps: Deps) -> StdResult<QueryLowBalancesResponse> {
    let infos: Vec<BalancesMappingQuery> = BALANCE_INFOS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| {
            let (k, v) = item?;
            Ok(BalancesMappingQuery {
                addr: k,
                label: v.label,
                assets: v.balances,
            })
        })
        .collect::<StdResult<_>>()?;

    let mut low_balance_assets: Vec<BalancesQuery> = vec![];

    for element in infos {
        let mut balance_query = BalancesQuery {
            addr: element.addr.clone(),
            label: element.label,
            assets: vec![],
        };
        for inner_element in element.assets {
            let result = query_balance(deps, element.addr.as_str(), &inner_element.asset)?;

            // only save into the list of balance query if balance amount is below the lower bound
            if result
                .mul(Uint128::from(10u64.pow(inner_element.decimals as u32)))
                .le(&inner_element.lower_bound)
            {
                balance_query.assets.push(Asset {
                    info: inner_element.asset,
                    amount: result,
                });
            }
        }

        // only append balance query into the list if we find an asset that has low balance
        if !balance_query.assets.is_empty() {
            low_balance_assets.push(balance_query);
        }
    }
    Ok(QueryLowBalancesResponse { low_balance_assets })
}

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::helpers::query_balance;
use crate::msg::{
    AddNewBalanceMsg, BalancesMappingQuery, BalancesQuery, DeleteBalanceMappingMsg,
    DeleteBalanceMsg, ExecuteMsg, InstantiateMsg, QueryBalanceMappingResponse,
    QueryBalancesMappingResponse, QueryLowBalancesResponse, QueryMsg, TopupMsg, UpdateBalanceMsg,
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
    ADMIN
        .assert_admin(deps.as_ref(), &info.sender)
        .map_err(|_| ContractError::InvalidAdmin {})?;
    let addr = deps.api.addr_validate(&msg.addr)?;

    // if already exist we append new balance into the list
    BALANCE_INFOS.update(
        deps.storage,
        addr,
        |balance_info| -> StdResult<BalanceInfo> {
            // if exist then we append the new balance into the list, else we create new
            if let Some(mut balance_info) = balance_info {
                // we dont allow repetitive balance info in the list to prevent spamming
                if balance_info
                    .balances
                    .clone()
                    .into_iter()
                    .find(|asset_data| asset_data.asset.eq(&msg.balance_info))
                    .is_some()
                {
                    return Err(StdError::generic_err(
                        ContractError::BalanceInfoExists {}.to_string(),
                    ));
                }
                balance_info.balances.push(AssetData {
                    asset: msg.balance_info.clone(),
                    lower_bound: msg.lower_bound,
                    upper_bound: msg.upper_bound,
                });
                return Ok(balance_info);
            }
            Ok(BalanceInfo {
                label: msg.label.unwrap_or_default(),
                balances: vec![AssetData {
                    asset: msg.balance_info.clone(),
                    lower_bound: msg.lower_bound,
                    upper_bound: msg.upper_bound,
                }],
            })
        },
    )?;
    // send response
    let res = Response::new()
        .add_attribute("action", "add_balance")
        .add_attribute("addr", msg.addr)
        .add_attribute("balance_info", msg.balance_info.to_string())
        .add_attribute("lower_bound", msg.lower_bound)
        .add_attribute("upper_bound", msg.upper_bound);
    Ok(res)
}

pub fn update_balance(
    deps: DepsMut,
    info: MessageInfo,
    msg: UpdateBalanceMsg,
) -> Result<Response, ContractError> {
    ADMIN
        .assert_admin(deps.as_ref(), &info.sender)
        .map_err(|_| ContractError::InvalidAdmin {})?;
    Err(ContractError::Std(StdError::generic_err("unimplemented")))
}

pub fn delete_balance(
    deps: DepsMut,
    info: MessageInfo,
    msg: DeleteBalanceMsg,
) -> Result<Response, ContractError> {
    ADMIN
        .assert_admin(deps.as_ref(), &info.sender)
        .map_err(|_| ContractError::InvalidAdmin {})?;
    Err(ContractError::Std(StdError::generic_err("unimplemented")))
}

pub fn update_balance_mapping(
    deps: DepsMut,
    info: MessageInfo,
    msg: DeleteBalanceMappingMsg,
) -> Result<Response, ContractError> {
    ADMIN
        .assert_admin(deps.as_ref(), &info.sender)
        .map_err(|_| ContractError::InvalidAdmin {})?;
    Err(ContractError::Std(StdError::generic_err("unimplemented")))
}

pub fn topup(deps: DepsMut, info: MessageInfo, msg: TopupMsg) -> Result<Response, ContractError> {
    Err(ContractError::Std(StdError::generic_err("unimplemented")))
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
            item.and_then(|(k, v)| {
                Ok(BalancesMappingQuery {
                    addr: k,
                    label: v.label,
                    assets: v.balances,
                })
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
            item.and_then(|(k, v)| {
                Ok(BalancesMappingQuery {
                    addr: k,
                    label: v.label,
                    assets: v.balances,
                })
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
            let result = query_balance(deps, element.addr.to_string(), inner_element.asset)?;

            // only save into the list of balance query if balance amount is below the lower bound
            if result.amount.le(&inner_element.lower_bound) {
                balance_query.assets.push(result);
            }
        }

        // only append balance query into the list if we find an asset that has low balance
        if !balance_query.assets.is_empty() {
            low_balance_assets.push(balance_query);
        }
    }
    Err(StdError::generic_err("unimplemented"))
}

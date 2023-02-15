use cosmwasm_std::{Deps, StdResult, Uint128};
use cw20::BalanceResponse;
use oraiswap::asset::AssetInfo;

pub fn query_balance(deps: Deps, address: &str, asset_info: &AssetInfo) -> StdResult<Uint128> {
    match asset_info.clone() {
        AssetInfo::NativeToken { denom } => {
            let response = deps.querier.query_balance(address, denom)?;
            return Ok(response.amount);
        }
        AssetInfo::Token { contract_addr } => {
            let response: BalanceResponse = deps.querier.query_wasm_smart(
                contract_addr,
                &cw20::Cw20QueryMsg::Balance {
                    address: address.into(),
                },
            )?;
            return Ok(response.balance);
        }
    }
}

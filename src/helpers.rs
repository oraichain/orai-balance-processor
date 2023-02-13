use cosmwasm_std::{Deps, StdResult};
use cw20::BalanceResponse;
use oraiswap::asset::{Asset, AssetInfo};

pub fn query_balance(deps: Deps, address: String, asset_info: AssetInfo) -> StdResult<Asset> {
    match asset_info.clone() {
        AssetInfo::NativeToken { denom } => {
            let response = deps.querier.query_balance(address, denom)?;
            return Ok(Asset {
                info: asset_info,
                amount: response.amount,
            });
        }
        AssetInfo::Token { contract_addr } => {
            let response: BalanceResponse = deps
                .querier
                .query_wasm_smart(contract_addr, &cw20::Cw20QueryMsg::Balance { address })?;
            return Ok(Asset {
                info: asset_info,
                amount: response.balance,
            });
        }
    }
}

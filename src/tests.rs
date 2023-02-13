use crate::contract::instantiate;
use crate::msg::InstantiateMsg;

use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::OwnedDeps;

pub fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies();

    // instantiate an empty contract
    let instantiate_msg = InstantiateMsg {};
    let admin = mock_info(&String::from("admin"), &[]);
    let res = instantiate(deps.as_mut(), mock_env(), admin, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    deps
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        from_binary,
        testing::{mock_env, mock_info},
        Addr, DepsMut, StdError, Uint128,
    };
    use cw_controllers::AdminResponse;
    use oraiswap::asset::{Asset, AssetInfo};

    use crate::{
        contract::{execute, query},
        msg::{
            AddNewBalanceMsg, DeleteBalanceMappingMsg, DeleteBalanceMsg, ExecuteMsg, QueryMsg,
            TopupMsg, UpdateBalanceMsg,
        },
        ContractError,
    };

    use super::setup;

    #[test]
    fn test_admin_query() {
        let mut deps = setup();
        let query_admin_msg = QueryMsg::QueryAdmin {};
        let response: AdminResponse =
            from_binary(&query(deps.as_ref(), mock_env(), query_admin_msg).unwrap()).unwrap();
        assert_eq!(response.admin, Some(String::from("admin")));
    }

    fn test_unauthorized_admin(deps: DepsMut, msg: ExecuteMsg) {
        let acc = mock_info(&String::from("unauthorized"), &[]);
        let response = execute(deps, mock_env(), acc, msg).unwrap_err();
        assert_eq!(
            response.to_string(),
            ContractError::InvalidAdmin {}.to_string()
        );
    }

    #[test]
    fn test_query_balances_mapping() {
        let mut deps = setup();
        let query_msg = QueryMsg::QueryBalancesMapping {};
        let response = query(deps.as_ref(), mock_env(), query_msg).unwrap_err();
        assert_eq!(response, StdError::generic_err("unimplemented"));
    }

    #[test]
    fn test_query_balance_mapping() {
        let mut deps = setup();
        let query_msg = QueryMsg::QueryBalanceMapping {
            addr: "foobar".to_string(),
        };
        let response = query(deps.as_ref(), mock_env(), query_msg).unwrap_err();
        assert_eq!(response, StdError::generic_err("unimplemented"));
    }

    #[test]
    fn test_query_low_balances() {
        let mut deps = setup();
        let query_msg = QueryMsg::QueryLowBalances {};
        let response = query(deps.as_ref(), mock_env(), query_msg).unwrap_err();
        assert_eq!(response, StdError::generic_err("unimplemented"));
    }

    #[test]
    fn test_add_balance() {
        let mut deps = setup();
        let asset_info = AssetInfo::NativeToken {
            denom: String::from("orai"),
        };
        let balance_info = AssetInfo::Token {
            contract_addr: Addr::unchecked("contract"),
        };
        let lower_bound = Uint128::from(50000u128);
        let upper_bound = Uint128::from(100000u128);
        let execute_msg = ExecuteMsg::AddBalance(AddNewBalanceMsg {
            asset_info,
            balance_info,
            lower_bound,
            upper_bound,
            label: Some("demo_balance".to_string()),
        });

        test_unauthorized_admin(deps.as_mut(), execute_msg.clone());

        let admin = mock_info(&String::from("admin"), &[]);
        let response = execute(deps.as_mut(), mock_env(), admin, execute_msg).unwrap_err();
        assert_eq!(
            response.to_string(),
            ContractError::Std(StdError::generic_err("unimplemented")).to_string()
        );
    }

    #[test]
    fn test_update_balance() {
        let mut deps = setup();
        let asset_info = AssetInfo::NativeToken {
            denom: String::from("orai"),
        };
        let balance_info = AssetInfo::Token {
            contract_addr: Addr::unchecked("contract"),
        };
        let lower_bound = Uint128::from(50000u128);
        let upper_bound = Uint128::from(100000u128);
        let execute_msg = ExecuteMsg::UpdateBalance(UpdateBalanceMsg {
            asset_info,
            balance_info,
            lower_bound,
            upper_bound,
            label: Some("demo_balance".to_string()),
        });

        test_unauthorized_admin(deps.as_mut(), execute_msg.clone());

        let admin = mock_info(&String::from("admin"), &[]);
        let response = execute(deps.as_mut(), mock_env(), admin, execute_msg).unwrap_err();
        assert_eq!(
            response.to_string(),
            ContractError::Std(StdError::generic_err("unimplemented")).to_string()
        );
    }

    #[test]
    fn test_delete_balance() {
        let mut deps = setup();
        let asset_info = AssetInfo::NativeToken {
            denom: String::from("orai"),
        };
        let balance_info = AssetInfo::Token {
            contract_addr: Addr::unchecked("contract"),
        };
        let execute_msg = ExecuteMsg::DeleteBalance(DeleteBalanceMsg {
            asset_info,
            balance_info,
        });

        test_unauthorized_admin(deps.as_mut(), execute_msg.clone());

        let admin = mock_info(&String::from("admin"), &[]);
        let response = execute(deps.as_mut(), mock_env(), admin, execute_msg).unwrap_err();
        assert_eq!(
            response.to_string(),
            ContractError::Std(StdError::generic_err("unimplemented")).to_string()
        );
    }

    #[test]
    fn test_delete_balance_mapping() {
        let mut deps = setup();
        let asset_info = AssetInfo::NativeToken {
            denom: String::from("orai"),
        };
        let execute_msg = ExecuteMsg::DeleteBalanceMapping(DeleteBalanceMappingMsg { asset_info });
        test_unauthorized_admin(deps.as_mut(), execute_msg.clone());
        let admin = mock_info(&String::from("admin"), &[]);
        let response = execute(deps.as_mut(), mock_env(), admin, execute_msg).unwrap_err();
        assert_eq!(
            response.to_string(),
            ContractError::Std(StdError::generic_err("unimplemented")).to_string()
        );
    }

    #[test]
    fn test_topup() {
        let mut deps = setup();
        let assets = vec![Asset {
            info: AssetInfo::NativeToken {
                denom: String::from("orai"),
            },
            amount: Uint128::from(10u128),
        }];
        let execute_msg = ExecuteMsg::Topup(TopupMsg { assets });
        let admin = mock_info(&String::from("admin"), &[]);
        let response = execute(deps.as_mut(), mock_env(), admin, execute_msg).unwrap_err();
        assert_eq!(
            response.to_string(),
            ContractError::Std(StdError::generic_err("unimplemented")).to_string()
        );
    }
}

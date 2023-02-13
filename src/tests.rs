use crate::contract::{execute, instantiate, query};
use crate::msg::InstantiateMsg;

use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{Addr, Empty, MessageInfo, OwnedDeps};
use cw20::MinterResponse;
use cw20_base::contract::{
    execute as execute_cw20, instantiate as instantiate_cw20, query as query_cw20,
};
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
use oraiswap::cw_multi_test::{App, Contract, ContractWrapper, Executor};

pub fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies();

    // instantiate an empty contract
    let instantiate_msg = InstantiateMsg {};
    let admin = mock_info(&String::from("admin"), &[]);
    let res = instantiate(deps.as_mut(), mock_env(), admin, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    deps
}

// setup multitest

fn mock_app() -> App {
    App::default()
}

fn contract_balance_processor() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query);
    Box::new(contract)
}

fn cw20() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute_cw20, instantiate_cw20, query_cw20);
    Box::new(contract)
}

fn init_multitest() -> (App, Addr, Addr, MessageInfo) {
    let mut router = mock_app();

    // init processor contract
    let contract_code_id = router.store_code(contract_balance_processor());
    let admin = mock_info(&String::from("admin"), &[]);

    let contract = router
        .instantiate_contract(
            contract_code_id,
            admin.sender.clone(),
            &InstantiateMsg {},
            &[],
            "processor",
            None,
        )
        .unwrap();

    // init cw20 contract
    let cw20_id = router.store_code(cw20());

    let cw20_contract = router
        .instantiate_contract(
            cw20_id,
            admin.sender.clone(),
            &Cw20InstantiateMsg {
                name: "USDT Token".to_string(),
                symbol: "USDT".to_string(),
                decimals: 6u8,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: admin.sender.to_string(),
                    cap: None,
                }),
                marketing: None,
            },
            &[],
            "processor",
            None,
        )
        .unwrap();

    (router, contract, cw20_contract, admin)
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        from_binary,
        testing::{mock_env, mock_info},
        Addr, DepsMut, StdError, Uint128,
    };
    use cw20::Cw20ExecuteMsg;
    use cw_controllers::AdminResponse;
    use oraiswap::{
        asset::{Asset, AssetInfo},
        cw_multi_test::Executor,
    };

    use crate::{
        contract::{execute, query},
        msg::{
            AddNewBalanceMsg, DeleteBalanceMappingMsg, DeleteBalanceMsg, ExecuteMsg,
            QueryBalanceMappingResponse, QueryMsg, TopupMsg, UpdateBalanceMsg,
        },
        tests::init_multitest,
        ContractError,
    };

    use super::setup;

    #[test]
    fn test_admin_query() {
        let deps = setup();
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
        let addr = "addr".to_string();
        let balance_info = AssetInfo::Token {
            contract_addr: Addr::unchecked("contract"),
        };
        let second_balance_info = AssetInfo::NativeToken {
            denom: "orai".to_string(),
        };
        let lower_bound = Uint128::from(50000u128);
        let upper_bound = Uint128::from(100000u128);
        let mut add_new_balance_msg = AddNewBalanceMsg {
            addr: addr.clone(),
            balance_info: balance_info.clone(),
            lower_bound,
            upper_bound,
            label: Some("demo_balance".to_string()),
        };
        let execute_msg = ExecuteMsg::AddBalance(add_new_balance_msg.clone());

        test_unauthorized_admin(deps.as_mut(), execute_msg.clone());

        let admin = mock_info(&String::from("admin"), &[]);
        let response = execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            execute_msg.clone(),
        )
        .unwrap();
        assert_eq!(response.attributes[1].value, addr);

        // if we try to add another same balance info => get error
        let response = execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            execute_msg.clone(),
        )
        .unwrap_err();
        assert_eq!(
            response.to_string(),
            StdError::generic_err(ContractError::BalanceInfoExists {}.to_string()).to_string()
        );

        // we can append to the list if balance info is different
        add_new_balance_msg.balance_info = second_balance_info.clone();
        let execute_msg = ExecuteMsg::AddBalance(add_new_balance_msg);
        execute(deps.as_mut(), mock_env(), admin, execute_msg).unwrap();

        // when querying it should show two balances info
        // query to double check if add balance is there
        let response: QueryBalanceMappingResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::QueryBalanceMapping { addr: addr.clone() },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            response.assets.last().unwrap().asset.to_string(),
            second_balance_info.to_string()
        );
        assert_eq!(
            response.assets.first().unwrap().asset.to_string(),
            balance_info.to_string()
        );
    }

    #[test]
    fn test_update_balance() {
        let mut deps = setup();
        let addr = "addr".to_string();
        let balance_info = AssetInfo::Token {
            contract_addr: Addr::unchecked("contract"),
        };
        let lower_bound = Uint128::from(50000u128);
        let upper_bound = Uint128::from(100000u128);
        let execute_msg = ExecuteMsg::UpdateBalance(UpdateBalanceMsg {
            addr,
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
        let addr = "addr".to_string();
        let balance_info = AssetInfo::Token {
            contract_addr: Addr::unchecked("contract"),
        };
        let execute_msg = ExecuteMsg::DeleteBalance(DeleteBalanceMsg { addr, balance_info });

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
        let addr = "addr".to_string();
        let execute_msg = ExecuteMsg::DeleteBalanceMapping(DeleteBalanceMappingMsg { addr });
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

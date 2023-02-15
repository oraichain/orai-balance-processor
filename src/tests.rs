use crate::contract::{execute, instantiate, query};
use crate::msg::InstantiateMsg;

use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{coins, Addr, BankMsg, Empty, MessageInfo, OwnedDeps};
use cw20::MinterResponse;
use cw20_base::contract::{
    execute as execute_cw20, instantiate as instantiate_cw20, query as query_cw20,
};
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
use oraiswap::cw_multi_test::{App, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};

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
    let admin = mock_info(&String::from("admin"), &coins(1000000u128, "orai"));

    // mint admin wallet orai so it can distribute to other wallets in test cases
    router
        .sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: admin.sender.clone().to_string(),
            amount: admin.clone().funds,
        }))
        .unwrap();

    let contract = router
        .instantiate_contract(
            contract_code_id,
            admin.sender.clone(),
            &InstantiateMsg {},
            &admin.funds,
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
        coins, from_binary,
        testing::{mock_env, mock_info},
        to_binary, Addr, BankMsg, BlockInfo, CosmosMsg, DepsMut, StdError, StdResult, Uint128,
        WasmMsg,
    };
    use cw20::Cw20ExecuteMsg;
    use cw_controllers::AdminResponse;
    use oraiswap::{
        asset::{Asset, AssetInfo},
        cw_multi_test::{BankSudo, Executor, SudoMsg},
    };

    use crate::{
        contract::{execute, query, MINIMUM_BLOCK_RANGE},
        msg::{
            AddNewBalanceMsg, BalancesQuery, DeleteBalanceMappingMsg, ExecuteMsg,
            QueryBalanceMappingResponse, QueryBalancesMappingResponse, QueryLowBalancesResponse,
            QueryMsg, TopupBalancesMsg, TopupMsg, UpdateBalanceMsg,
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
        let addr = "addr".to_string();
        let second_addr = "second_addr".to_string();
        let query_msg = QueryMsg::QueryBalancesMapping {};

        // should be empty at first
        let response: QueryBalancesMappingResponse =
            from_binary(&query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap()).unwrap();
        assert_eq!(response.balance_assets.len(), 0usize);

        // when adding a new balance info, should query new data
        let balance_info = AssetInfo::Token {
            contract_addr: Addr::unchecked("contract"),
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
        execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            execute_msg.clone(),
        )
        .unwrap();

        // add a second balance mapping pair to test query
        add_new_balance_msg.addr = second_addr.clone();
        let execute_msg = ExecuteMsg::AddBalance(add_new_balance_msg.clone());
        execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            execute_msg.clone(),
        )
        .unwrap();

        // when querying it should show two balances info
        // query to double check if add balance is there
        let response: QueryBalancesMappingResponse = from_binary(
            &query(deps.as_ref(), mock_env(), QueryMsg::QueryBalancesMapping {}).unwrap(),
        )
        .unwrap();
        assert_eq!(response.balance_assets.first().unwrap().addr, addr.clone());
        assert_eq!(
            response.balance_assets.last().unwrap().addr,
            second_addr.clone()
        );
    }

    #[test]
    fn test_query_balance_mapping() {
        let mut deps = setup();
        let addr = "addr".to_string();
        let query_msg = QueryMsg::QueryBalanceMapping { addr: addr.clone() };

        // should be empty at first
        let response = &query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap_err();
        assert_eq!(
            response.to_string(),
            StdError::NotFound {
                kind: "oraiswap_balance_processor::state::BalanceInfo".to_string()
            }
            .to_string()
        );

        // when adding a new balance info, should query new data
        let balance_info = AssetInfo::Token {
            contract_addr: Addr::unchecked("contract"),
        };
        let lower_bound = Uint128::from(50000u128);
        let upper_bound = Uint128::from(100000u128);
        let add_new_balance_msg = AddNewBalanceMsg {
            addr: addr.clone(),
            balance_info: balance_info.clone(),
            lower_bound,
            upper_bound,
            label: Some("demo_balance".to_string()),
        };
        let execute_msg = ExecuteMsg::AddBalance(add_new_balance_msg.clone());
        test_unauthorized_admin(deps.as_mut(), execute_msg.clone());
        let admin = mock_info(&String::from("admin"), &[]);
        execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            execute_msg.clone(),
        )
        .unwrap();

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
            balance_info.to_string()
        );
    }

    #[test]
    fn test_query_low_balances() {
        let (mut deps, addr, cw20_addr, admin) = init_multitest();
        let mock_addr = mock_info("sender", &vec![]);
        let native_balance_info_denom = "orai".to_string();
        let cw20_balance_info_address = cw20_addr.to_string();
        let admin_addr = admin.sender;
        // init msgs to send the admin addr some cw20 & native tokens
        deps.execute(
            addr.clone(),
            BankMsg::Send {
                to_address: mock_addr.sender.clone().into_string(),
                amount: coins(10u128, native_balance_info_denom.clone()),
            }
            .into(),
        )
        .unwrap();

        deps.execute_contract(
            admin_addr.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::Mint {
                recipient: mock_addr.sender.clone().to_string(),
                amount: Uint128::from(100u128),
            },
            &vec![],
        )
        .unwrap();

        // store two balance infos above into the balance mapping to try querying low balances
        // store native balance info orai
        deps.execute_contract(
            admin_addr.clone(),
            addr.clone(),
            &ExecuteMsg::AddBalance(AddNewBalanceMsg {
                addr: mock_addr.sender.clone().to_string(),
                balance_info: AssetInfo::NativeToken {
                    denom: native_balance_info_denom.clone(),
                },
                lower_bound: Uint128::from(11u128), // current balance is 10u128, should trigger low balance
                upper_bound: Uint128::from(100u128),
                label: Some("demo_balance".to_string()),
            }),
            &vec![],
        )
        .unwrap();

        // store cw20 balance
        deps.execute_contract(
            admin_addr.clone(),
            addr.clone(),
            &ExecuteMsg::AddBalance(AddNewBalanceMsg {
                addr: mock_addr.sender.clone().to_string(),
                balance_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked(&cw20_balance_info_address.clone()),
                },
                lower_bound: Uint128::from(11u128), // current balance is 10u128, should trigger low balance
                upper_bound: Uint128::from(1000u128),
                label: Some("demo_balance".to_string()),
            }),
            &vec![],
        )
        .unwrap();

        // query low balance, should return only native balance because it is lower than lower bound
        let response: QueryLowBalancesResponse = deps
            .wrap()
            .query_wasm_smart(addr.clone().into_string(), &QueryMsg::QueryLowBalances {})
            .unwrap();
        assert_eq!(
            response
                .low_balance_assets
                .last()
                .unwrap()
                .assets
                .last()
                .unwrap()
                .info,
            AssetInfo::NativeToken {
                denom: native_balance_info_denom.clone()
            }
        );

        // if now we top up mock addr with native address, the list should be empty because lower bound of native is 11u128
        deps.execute(
            addr.clone(),
            BankMsg::Send {
                to_address: mock_addr.sender.clone().to_string(),
                amount: coins(100u128, native_balance_info_denom.clone()),
            }
            .into(),
        )
        .unwrap();

        let response: QueryLowBalancesResponse = deps
            .wrap()
            .query_wasm_smart(addr.clone().into_string(), &QueryMsg::QueryLowBalances {})
            .unwrap();
        assert_eq!(response.low_balance_assets.len(), 0usize);
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
        let second_balance_info = AssetInfo::NativeToken {
            denom: "orai".to_string(),
        };
        let lower_bound = Uint128::from(50000u128);
        let upper_bound = Uint128::from(100000u128);
        let mut add_new_balance_msg = AddNewBalanceMsg {
            addr: addr.clone(),
            balance_info: balance_info.clone(),
            lower_bound: Uint128::from(1u128),
            upper_bound: Uint128::from(10u128),
            label: Some("demo_balance".to_string()),
        };
        let execute_msg = ExecuteMsg::AddBalance(add_new_balance_msg.clone());
        let admin = mock_info(&String::from("admin"), &[]);
        // add new balance mapping first before updating it
        execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            execute_msg.clone(),
        )
        .unwrap();
        // Add another balance mapping so we can observe the difference when we update for an existing balance info
        add_new_balance_msg.balance_info = second_balance_info.clone();
        let execute_msg = ExecuteMsg::AddBalance(add_new_balance_msg);
        execute(deps.as_mut(), mock_env(), admin, execute_msg).unwrap();

        // now we try to update the balance to new lower & upper bound
        let execute_msg = ExecuteMsg::UpdateBalance(UpdateBalanceMsg {
            addr: addr.clone(),
            balance_info: balance_info.clone(),
            lower_bound: Some(lower_bound),
            upper_bound: Some(upper_bound),
        });
        test_unauthorized_admin(deps.as_mut(), execute_msg.clone());
        let admin = mock_info(&String::from("admin"), &[]);
        execute(deps.as_mut(), mock_env(), admin, execute_msg).unwrap();

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
        assert_eq!(response.assets[0].lower_bound, lower_bound); // asset info {"contract_addr":"contract"} lower bound & upper bound should be updated

        // balance mapping not exist case
        // now we try to update the balance to new lower & upper bound
        let execute_msg = ExecuteMsg::UpdateBalance(UpdateBalanceMsg {
            addr: "not-exist".to_string(),
            balance_info: balance_info.clone(),
            lower_bound: Some(lower_bound),
            upper_bound: Some(upper_bound),
        });
        let admin = mock_info(&String::from("admin"), &[]);
        let response_err = execute(deps.as_mut(), mock_env(), admin, execute_msg).unwrap_err();
        assert_eq!(
            response_err.to_string(),
            StdError::generic_err(ContractError::BalanceMappingNotExist {}.to_string()).to_string()
        );

        // Balance info not exist case
        let execute_msg = ExecuteMsg::UpdateBalance(UpdateBalanceMsg {
            addr: addr.clone(),
            balance_info: AssetInfo::Token {
                contract_addr: Addr::unchecked("not-exist"),
            },
            lower_bound: Some(lower_bound),
            upper_bound: Some(upper_bound),
        });
        let admin = mock_info(&String::from("admin"), &[]);
        let response_err = execute(deps.as_mut(), mock_env(), admin, execute_msg).unwrap_err();
        assert_eq!(
            response_err.to_string(),
            StdError::generic_err(ContractError::BalanceInfoNotExist {}.to_string()).to_string()
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
        let (mut deps, addr, cw20_addr, admin) = init_multitest();
        let mock_addr = mock_info("sender", &vec![]);
        let native_balance_info_denom = "orai".to_string();
        let cw20_balance_info_address = cw20_addr.to_string();
        let admin_addr = admin.sender;
        let native_balance_info_upper_bound = Uint128::from(100u128);
        // init msgs to send the admin addr some cw20 & native tokens
        deps.execute(
            addr.clone(),
            BankMsg::Send {
                to_address: mock_addr.sender.clone().into_string(),
                amount: coins(10u128, native_balance_info_denom.clone()),
            }
            .into(),
        )
        .unwrap();

        // mint new cw20 addr to mock addr
        deps.execute_contract(
            admin_addr.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::Mint {
                recipient: mock_addr.sender.clone().to_string(),
                amount: Uint128::from(100u128),
            },
            &vec![],
        )
        .unwrap();

        // store two balance infos above into the balance mapping to try querying low balances
        // store native balance info orai
        deps.execute_contract(
            admin_addr.clone(),
            addr.clone(),
            &ExecuteMsg::AddBalance(AddNewBalanceMsg {
                addr: mock_addr.sender.clone().to_string(),
                balance_info: AssetInfo::NativeToken {
                    denom: native_balance_info_denom.clone(),
                },
                lower_bound: Uint128::from(11u128), // current balance is 10u128, should trigger low balance
                upper_bound: native_balance_info_upper_bound.clone(),
                label: Some("demo_balance".to_string()),
            }),
            &vec![],
        )
        .unwrap();

        // store cw20 balance
        deps.execute_contract(
            admin_addr.clone(),
            addr.clone(),
            &ExecuteMsg::AddBalance(AddNewBalanceMsg {
                addr: mock_addr.sender.clone().to_string(),
                balance_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked(&cw20_balance_info_address.clone()),
                },
                lower_bound: Uint128::from(11u128), // current balance is 10u128, should trigger low balance
                upper_bound: Uint128::from(1000u128),
                label: Some("demo_balance".to_string()),
            }),
            &vec![],
        )
        .unwrap();

        // mint new not-in-list-denom to mock addr, this would help pass the non-existed error
        deps.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: admin_addr.clone().to_string(),
            amount: coins(1u128, "not-in-list-denom"),
        }))
        .unwrap();

        // before top-up, orai denom is in low balance for mock addr
        let response: QueryLowBalancesResponse = deps
            .wrap()
            .query_wasm_smart(addr.clone().into_string(), &QueryMsg::QueryLowBalances {})
            .unwrap();
        assert_eq!(response.low_balance_assets.len(), 1usize);
        assert_eq!(
            response.low_balance_assets[0].assets[0].info,
            AssetInfo::NativeToken {
                denom: native_balance_info_denom.clone().to_string()
            }
        );

        let balances = vec![
            // non existed denom case
            // top up msg includes wrong asset case, should not top-up because the asset is not in the list
            TopupBalancesMsg {
                addr: mock_addr.sender.clone(),
                asset_infos: vec![AssetInfo::NativeToken {
                    denom: "non-existed".to_string(),
                }],
            },
            // not in the list of balance mapping case
            TopupBalancesMsg {
                addr: mock_addr.sender.clone(),
                asset_infos: vec![AssetInfo::NativeToken {
                    denom: "not-in-list-denom".to_string(),
                }],
            },
            // balance is not below lower bound
            TopupBalancesMsg {
                addr: mock_addr.sender.clone(),
                asset_infos: vec![AssetInfo::Token {
                    contract_addr: cw20_addr.clone(),
                }],
            },
            // correct case which is orai low balance, should top up
            TopupBalancesMsg {
                addr: mock_addr.sender.clone(),
                asset_infos: vec![
                    AssetInfo::NativeToken {
                        denom: native_balance_info_denom.to_string(),
                    },
                    // multiple same asset info in a topup balance msg wont work, will only mark as one topup msg
                    AssetInfo::NativeToken {
                        denom: native_balance_info_denom.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: native_balance_info_denom.to_string(),
                    },
                    AssetInfo::NativeToken {
                        denom: native_balance_info_denom.to_string(),
                    },
                ],
            },
            // multiple topup balance msg with same addr & asset info wont work, will only mark as one topup msg
            TopupBalancesMsg {
                addr: mock_addr.sender.clone(),
                asset_infos: vec![AssetInfo::NativeToken {
                    denom: native_balance_info_denom.to_string(),
                }],
            },
        ];
        let execute_msg = ExecuteMsg::Topup(TopupMsg { balances });
        let admin = mock_info(&String::from("admin"), &[]);
        let response = deps
            .execute_contract(
                admin.sender.clone(),
                addr.clone(),
                &execute_msg.clone(),
                &vec![],
            )
            .unwrap();
        assert_eq!(response.events[1].attributes.last().unwrap().value, "1");
        assert_eq!(response.events[1].attributes[2].value, "true");

        // after topping up, should not have any low balance
        let response: QueryLowBalancesResponse = deps
            .wrap()
            .query_wasm_smart(addr.clone().into_string(), &QueryMsg::QueryLowBalances {})
            .unwrap();
        assert_eq!(response.low_balance_assets.len(), 0usize);

        // wallet hack case. Asset is drained, and hacker will call multiple top-up txs in a short period. Should reject & only allow top-up once per day for an asset
        // pretend that the mock addr is hacked & sent all top-up tokens back to contract
        deps.execute(
            mock_addr.sender.clone(),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: addr.clone().into_string(),
                amount: coins(
                    native_balance_info_upper_bound.u128(),
                    native_balance_info_denom.clone(),
                ),
            }),
        )
        .unwrap();
        // now we try to top-up again immediately afterwards => should fail
        let response = deps
            .execute_contract(
                admin.sender.clone(),
                addr.clone(),
                &execute_msg.clone(),
                &vec![],
            )
            .unwrap();
        assert_eq!(response.events[1].attributes.last().unwrap().value, "0");
        assert_eq!(response.events[1].attributes[2].value, "true");

        // we need to wait for at least a day to re-topup
        let mut block_info = deps.block_info();
        block_info.height += MINIMUM_BLOCK_RANGE + 1;
        deps.set_block(block_info);

        // now we can top-up
        let response = deps
            .execute_contract(
                admin.sender.clone(),
                addr.clone(),
                &execute_msg.clone(),
                &vec![],
            )
            .unwrap();
        assert_eq!(response.events[1].attributes.last().unwrap().value, "1");
    }
}

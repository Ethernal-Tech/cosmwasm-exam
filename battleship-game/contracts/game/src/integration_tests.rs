#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use cosmwasm_std::{Addr, Uint128};
    use cw20::{Cw20QueryMsg, BalanceResponse};
    use cw_multi_test::{App, ContractWrapper, Executor, IntoAddr};
    use cw20_base::contract::{instantiate as cw20_instantiate, execute as cw20_execute, query as cw20_query};
    use cw20_base::msg::{InstantiateMsg as Cw20InstantiateMsg, ExecuteMsg as Cw20ExecuteMsg};
    use crate::msg::{BoolResponse, ProofStep};
    use crate::{
        contract::{execute, instantiate, query}, 
        msg::{
            AdminResponse, ExecuteMsg, InstantiateMsg, PlayerInstantiate, QueryMsg, ShipsResponse, AddressResponse
        }, state::Player, ContractError
    };

    pub fn mock_instantiate_msg(ships: usize, token_address: Addr) -> InstantiateMsg {
        InstantiateMsg {
            admin: "admin".into_addr().to_string(),
            ships: ships,
            token_address: token_address.to_string(),
            players: vec![
                PlayerInstantiate {
                    address: "player1".into_addr().to_string(),
                    stake: Uint128::new(1000),
                    board: "372ecd2044c797715c6d02c9f5b0fd2594172620a632e8a7dd3f10bfa8f2df56".to_owned(),
                },
                PlayerInstantiate {
                    address: "player2".into_addr().to_string(),
                    stake: Uint128::new(1000),
                    board: "ee5fd4795e374a9e78f447867419b5ea98e662383c0ee88622e00cc7f710165c".to_owned(),
                },
            ],
        }
    }

    pub fn mock_cw20_instantiate_msg(
        player1_addr: Addr, 
        player2_addr: Addr,
        admin_addr: Addr
    ) -> Cw20InstantiateMsg {
        Cw20InstantiateMsg {
            name: "BattleToken".to_string(),
            symbol: "BTL".to_string(),
            decimals: 6,
            initial_balances: vec![
                cw20::Cw20Coin {
                    address: player1_addr.to_string(),
                    amount: Uint128::new(1_000_000),
                },
                cw20::Cw20Coin {
                    address: player2_addr.to_string(),
                    amount: Uint128::new(1_000_000),
                },
            ],
            mint: Some(cw20::MinterResponse {
                minter: admin_addr.to_string(),
                cap: None,
            }),
            marketing: None,
        }
    }

    pub fn init_app(player1_addr: Addr, player2_addr: Addr) -> (Addr, Addr, App) {
        let mut app = App::default();

        let cw20_code = ContractWrapper::new(cw20_execute, cw20_instantiate, cw20_query);
        let cw20_code_id = app.store_code(Box::new(cw20_code));

        let admin_addr = "admin".into_addr();

        let cw20_addr = app
            .instantiate_contract(
                cw20_code_id, 
                "owner".into_addr(), 
                &mock_cw20_instantiate_msg(
                    player1_addr.clone(), 
                    player2_addr.clone(), 
                    admin_addr.clone()
                ), 
                &[], 
                "cw20-token", 
                None)
            .unwrap();

        let game_code = ContractWrapper::new(execute, instantiate, query);
        let game_code_id = app.store_code(Box::new(game_code));

        let game_addr = app
            .instantiate_contract(
                game_code_id, 
                "owner".into_addr(),
                &mock_instantiate_msg(1, cw20_addr.clone()), 
                &[], 
                "Contract", 
                None
        ).unwrap();

        app.execute_contract(
            admin_addr,
            cw20_addr.clone(),
            &Cw20ExecuteMsg::UpdateMinter {
                new_minter: Some(game_addr.to_string()),
            },
            &[],
        ).unwrap();

        app.execute_contract(
            player1_addr,
            cw20_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance { 
                spender: game_addr.to_string(),
                amount: Uint128::new(100_000), 
                expires: None 
            },
            &[]
        ).unwrap();

        app.execute_contract(
            player2_addr,
            cw20_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance { 
                spender: game_addr.to_string(),
                amount: Uint128::new(100_000), 
                expires: None 
            },
            &[]
        ).unwrap();

        (cw20_addr, game_addr, app)
    }

    #[test]
    fn instantiation() {
        let player1_addr = "player1".into_addr();
        let player2_addr = "player2".into_addr();
        let (cw20_address, address, app) = init_app(player1_addr, player2_addr);

        let response: AdminResponse = app
            .wrap()
            .query_wasm_smart(address.clone(), &QueryMsg::GetAdmin {  })
            .unwrap();

        assert_eq!(response.admin, "admin".into_addr());

        let response: ShipsResponse = app
            .wrap()
            .query_wasm_smart(address.clone(), &QueryMsg::GetShips {})
            .unwrap();

        assert_eq!(response.ships, 1);

        let response: AddressResponse = app
            .wrap()
            .query_wasm_smart(address.clone(), &QueryMsg::GetTurn {  })
            .unwrap();

        assert_eq!(response.address, "player1".into_addr());

        let response: AddressResponse = app
            .wrap()
            .query_wasm_smart(address.clone(), &QueryMsg::GetTokenAddress {  })
            .unwrap();

        assert_eq!(response.address, cw20_address);

        let response: BoolResponse = app
            .wrap()
            .query_wasm_smart(address.clone(), &QueryMsg::GetStarted {  })
            .unwrap();

        assert_eq!(response.value, false);

        let response: BoolResponse = app
            .wrap()
            .query_wasm_smart(address.clone(), &QueryMsg::GetFinished {  })
            .unwrap();

        assert_eq!(response.value, false);

        let response: Vec<Player> = app
            .wrap()
            .query_wasm_smart(address.clone(), &QueryMsg::GetPlayers {  })
            .unwrap();

        assert_eq!(response[1].address, "player1".into_addr());
        assert_eq!(response[1].stake, Uint128::new(1000));
        assert_eq!(response[1].board.fields, "372ecd2044c797715c6d02c9f5b0fd2594172620a632e8a7dd3f10bfa8f2df56".to_owned(),);
        assert_eq!(response[1].board.sank, vec![]);

        assert_eq!(response[0].address, "player2".into_addr());
        assert_eq!(response[0].stake, Uint128::new(1000));
        assert_eq!(response[0].board.fields, "ee5fd4795e374a9e78f447867419b5ea98e662383c0ee88622e00cc7f710165c".to_owned());
        assert_eq!(response[0].board.sank, vec![]);
    }

    #[test]
    fn should_throw_invalid_ships_error() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let err = app
            .instantiate_contract(
                code_id, 
                "owner".into_addr(),
                &mock_instantiate_msg(0, "token".into_addr()), 
                &[], 
                "Contract", 
                None
            ).unwrap_err();

        assert_eq!(ContractError::InvalidShips {  }, err.downcast().unwrap())
    }

    // #[test]
    // fn should_throw_invalid_board_error() {
    //     let mut app = App::default();

    //     let code = ContractWrapper::new(execute, instantiate, query);
    //     let code_id = app.store_code(Box::new(code));

    //     let err = app
    //         .instantiate_contract(
    //             code_id, 
    //             "owner".into_addr(),
    //             &mock_instantiate_msg(3, "token".into_addr()), 
    //             &[], 
    //             "Contract", 
    //             None
    //         ).unwrap_err();

    //     assert_eq!(ContractError::InvalidBoard {  }, err.downcast().unwrap())
    // }

    #[test]
    fn game() {
        let player1_addr = "player1".into_addr();
        let player2_addr = "player2".into_addr();
        let (cw20_addr, game_addr, mut app) = init_app(
            player1_addr.clone(),
            player2_addr.clone()
        );

        // start game
        let response = app
            .execute_contract(
                player1_addr.clone(), 
                game_addr.clone(), 
                &ExecuteMsg::StartGame {}, 
                &[]
        ).unwrap();

        let wasm = response
            .events.iter()
            .find(|ev| ev.ty == "wasm")
            .unwrap();
        assert_eq!(
            wasm.attributes
                .iter()
                .find(|attr| attr.key == "action")
                .unwrap()
                .value,
            "start_game"
        );
        assert_eq!(
            Uint128::from_str(&wasm.attributes
                .iter()
                .find(|attr| attr.key == "stake")
                .unwrap()
                .value 
            ).unwrap(),
            Uint128::new(1000)
        );

        let contract_balance: BalanceResponse = app.wrap()
            .query_wasm_smart(
                cw20_addr.clone(),
                &Cw20QueryMsg::Balance { 
                    address: game_addr.to_string() 
                } 
        ).unwrap();
        assert_eq!(contract_balance.balance, Uint128::new(2000));

        let player1_balance: BalanceResponse = app.wrap()
            .query_wasm_smart(
                cw20_addr.clone(),
                &Cw20QueryMsg::Balance {
                    address: player1_addr.clone().to_string(),
                },
        ).unwrap();
        assert_eq!(player1_balance.balance, Uint128::new(1000000 - 1000));

        let player1_balance: BalanceResponse = app.wrap()
            .query_wasm_smart(
                cw20_addr.clone(),
                &Cw20QueryMsg::Balance {
                    address: player2_addr.clone().to_string(),
                },
        ).unwrap();
        assert_eq!(player1_balance.balance, Uint128::new(1000000 - 1000));

        // player1's turn
        let response = app
            .execute_contract(
                "player1".into_addr(),
                game_addr.clone(),
                &ExecuteMsg::Play { 
                    field: (1, 0),
                    value: false,
                    proof: vec![
                        ProofStep {
                            hash: "fcbcf165908dd18a9e49f7ff27810176db8e9f63b4352213741664245224f8aa".to_owned(),
                            is_left: true
                        },
                        ProofStep {
                            hash: "b39595dabdf67f2d2b1c22e6690c8500c89ccb9d817f1cce4b47337910cbe2cb".to_owned(),
                            is_left: true
                        },
                        ProofStep {
                            hash: "9a74c4cb6669420048ea661a3b8e501ca5c10c2f2680a41acc2128422c1ff6b6".to_owned(),
                            is_left: false
                        },
                        ProofStep {
                            hash: "2e176322075537fd763b4405a4392854d0f67b079f814bbe9d9683ca371343e9".to_owned(),
                            is_left: false
                        },
                    ] 
                },
                &[]
            )
            .unwrap();

        let wasm = response
            .events.iter()
            .find(|ev| ev.ty == "wasm")
            .unwrap();
        assert_eq!(
            wasm.attributes
                .iter()
                .find(|attr| attr.key == "action")
                .unwrap()
                .value,
            "play"
        );
        
        let ship_missed: Vec<_> = response
            .events
            .iter()
            .filter(|ev| ev.ty == "wasm-ship_missed")
            .collect();
        assert_eq!(
            ship_missed[0]
                .attributes
                .iter()
                .find(|attr| attr.key == "missed")
                .unwrap()
                .value,
            stringify!((1, 0))
        );

        // player2's turn
        let response = app
            .execute_contract(
                "player2".into_addr(),
                game_addr.clone(),
                &ExecuteMsg::Play { 
                    field: (1, 1),
                    value: true,
                    proof: vec![
                        ProofStep {
                            hash: "fcbcf165908dd18a9e49f7ff27810176db8e9f63b4352213741664245224f8aa".to_owned(),
                            is_left: false
                        },
                        ProofStep {
                            hash: "33873cc7849b5a36e72508f177d655726b17aa5adeded878804cc402ae1ecbc1".to_owned(),
                            is_left: false
                        },
                        ProofStep {
                            hash: "9a74c4cb6669420048ea661a3b8e501ca5c10c2f2680a41acc2128422c1ff6b6".to_owned(),
                            is_left: true
                        },
                        ProofStep {
                            hash: "2e176322075537fd763b4405a4392854d0f67b079f814bbe9d9683ca371343e9".to_owned(),
                            is_left: false
                        }
                    ]
                },
                &[]
            )
            .unwrap();

        let wasm = response
            .events.iter()
            .find(|ev| ev.ty == "wasm")
            .unwrap();
        assert_eq!(
            wasm.attributes
                .iter()
                .find(|attr| attr.key == "action")
                .unwrap()
                .value,
            "play"
        );

        let wasm = response
            .events.iter()
            .find(|ev| ev.ty == "wasm")
            .unwrap();
        assert_eq!(
            wasm.attributes
                .iter()
                .find(|attr| attr.key == "winner")
                .unwrap()
                .value,
            player2_addr.clone().to_string()
        );

        assert_eq!(
            Uint128::from_str(
                &wasm.attributes
                .iter()
                .find(|a| a.key == "payout")
                .unwrap()
                .value
            ).unwrap(),
            Uint128::new(1900)
        );

        assert_eq!(
            Uint128::from_str(
                &wasm.attributes
                .iter()
                .find(|a| a.key == "fee_retained")
                .unwrap()
                .value
            ).unwrap(),
            Uint128::new(100)
        );

        assert_eq!(
            Uint128::from_str(
                &wasm.attributes
                .iter()
                .find(|a| a.key == "minted_reward")
                .unwrap()
                .value
            ).unwrap(),
            Uint128::new(19)
        );

        let game_won: Vec<_> = response
            .events
            .iter()
            .filter(|ev| ev.ty == "wasm-game_won")
            .collect();
        assert_eq!(
            game_won[0]
                .attributes
                .iter()
                .find(|attr| attr.key == "sank")
                .unwrap()
                .value,
            stringify!((1, 1))
        );

        let contract_balance: cw20::BalanceResponse = app.wrap()
            .query_wasm_smart(
                cw20_addr.clone(),
                &Cw20QueryMsg::Balance {
                    address: game_addr.to_string(),
            }
        ).unwrap();
        assert_eq!(contract_balance.balance, Uint128::new(100));

        let winner_balance: cw20::BalanceResponse = app.wrap()
            .query_wasm_smart(
                cw20_addr.clone(),
                &Cw20QueryMsg::Balance {
                    address: player2_addr.to_string(),
        },
        )
        .unwrap();

        assert_eq!(winner_balance.balance, Uint128::new(1_000_000 - 1_000 + 1919));

        let token_info: cw20::TokenInfoResponse = app.wrap()
            .query_wasm_smart(
                cw20_addr.clone(),
                &Cw20QueryMsg::TokenInfo {},
            )
        .unwrap();

        assert_eq!(token_info.total_supply, Uint128::new(2_000_000 + 19));

        let err = app
            .execute_contract(
                "player1".into_addr(),
                game_addr.clone(),
                &ExecuteMsg::Play {
                    field: (1, 0),
                    value: false,
                    proof: vec![]
                },
                &[]
            )
            .unwrap_err();

        assert_eq!(ContractError::GameFinished {  }, err.downcast().unwrap())

    }

    #[test]
    fn should_throw_wrong_turn_error() {
        let player1_addr = "player1".into_addr();
        let player2_addr = "player2".into_addr();
        let (_, game_addr, mut app) = init_app(player1_addr.clone(), player2_addr.clone());

        app
            .execute_contract(
                player1_addr.clone(), 
                game_addr.clone(), 
                &ExecuteMsg::StartGame {}, 
                &[]
        ).unwrap();

        let err = app
            .execute_contract(
                "player2".into_addr(),
                game_addr.clone(),
                &ExecuteMsg::Play {
                    field: (1, 0),
                    value: false,
                    proof: vec![]
                },
                &[]
            )
            .unwrap_err();

        assert_eq!(ContractError::WrongTurn {  }, err.downcast().unwrap())
    }

    #[test]
    fn should_throw_game_started_error() {
        let player1_addr = "player1".into_addr();
        let player2_addr = "player2".into_addr();
        let (_, game_addr, mut app) = init_app(player1_addr.clone(), player2_addr.clone());

        let _ = app
            .execute_contract(
                player1_addr.clone(), 
                game_addr.clone(), 
                &ExecuteMsg::StartGame {}, 
                &[]
        ).unwrap();

        let err = app
            .execute_contract(
                player1_addr.clone(), 
                game_addr.clone(), 
                &ExecuteMsg::StartGame {}, 
                &[]
        ).unwrap_err();
        assert_eq!(ContractError::GameStarted {  }, err.downcast().unwrap())
    }

    #[test]
    fn should_throw_turn_expired_error() {
        let player1_addr = "player1".into_addr();
        let player2_addr = "player2".into_addr();
        let (_, game_addr, mut app) = init_app(
            player1_addr.clone(),
            player2_addr.clone()
        );

        // start game
        app
            .execute_contract(
                player1_addr.clone(), 
                game_addr.clone(), 
                &ExecuteMsg::StartGame {}, 
                &[]
        ).unwrap();

        app.update_block(|b| b.time = b.time.plus_seconds(1000));

        let err = app
            .execute_contract(
                "player1".into_addr(),
                game_addr.clone(),
                &ExecuteMsg::Play {
                    field: (1, 0),
                    value: false,
                    proof: vec![]
                },
                &[]
            )
            .unwrap_err();
        assert_eq!(ContractError::TurnExpired {  }, err.downcast().unwrap());
    }

    #[test]
    fn should_throw_unauthorized_error() {
        let player1_addr = "player1".into_addr();
        let player2_addr = "player2".into_addr();
        let (_, game_addr, mut app) = init_app(player1_addr.clone(), player2_addr.clone());

        let err = app
            .execute_contract(
                "attacker".into_addr(), 
                game_addr.clone(), 
                &ExecuteMsg::StartGame {}, 
                &[]
        ).unwrap_err();

        assert_eq!(ContractError::Unauthorized {  }, err.downcast().unwrap())
    }

    #[test]
    fn should_throw_game_not_started_error() {
        let player1_addr = "player1".into_addr();
        let player2_addr = "player2".into_addr();
        let (_, game_addr, mut app) = init_app(player1_addr.clone(), player2_addr.clone());

        let err = app
            .execute_contract(
                "player2".into_addr(),
                game_addr.clone(),
                &ExecuteMsg::Play {
                    field: (1, 0),
                    value: false,
                    proof: vec![]
                },
                &[]
            )
            .unwrap_err();

        assert_eq!(ContractError::GameNotStarted {  }, err.downcast().unwrap())
    }

    #[test]
    fn timeout_win() {
        let player1_addr = "player1".into_addr();
        let player2_addr = "player2".into_addr();
        let (cw20_addr, game_addr, mut app) = init_app(
            player1_addr.clone(),
            player2_addr.clone()
        );

        // start game
        app
            .execute_contract(
                player1_addr.clone(), 
                game_addr.clone(), 
                &ExecuteMsg::StartGame {}, 
                &[]
        ).unwrap();

        app.update_block(|b| b.time = b.time.plus_seconds(1000));

        let response = app.execute_contract(
            player2_addr.clone(),
            game_addr.clone(),
            &ExecuteMsg::TimeoutWin {},
            &[]
        ).unwrap();

        let wasm = response
            .events.iter()
            .find(|ev| ev.ty == "wasm")
            .unwrap();
        assert_eq!(
            wasm.attributes
                .iter()
                .find(|attr| attr.key == "action")
                .unwrap()
                .value,
            "timeout_check"
        );

        let wasm = response
            .events.iter()
            .find(|ev| ev.ty == "wasm")
            .unwrap();
        assert_eq!(
            wasm.attributes
                .iter()
                .find(|attr| attr.key == "winner")
                .unwrap()
                .value,
            player2_addr.clone().to_string()
        );

        assert_eq!(
            Uint128::from_str(
                &wasm.attributes
                .iter()
                .find(|a| a.key == "payout")
                .unwrap()
                .value
            ).unwrap(),
            Uint128::new(1900)
        );

        assert_eq!(
            Uint128::from_str(
                &wasm.attributes
                .iter()
                .find(|a| a.key == "fee_retained")
                .unwrap()
                .value
            ).unwrap(),
            Uint128::new(100)
        );

        assert_eq!(
            Uint128::from_str(
                &wasm.attributes
                .iter()
                .find(|a| a.key == "minted_reward")
                .unwrap()
                .value
            ).unwrap(),
            Uint128::new(19)
        );

        let game_won: Vec<_> = response
            .events
            .iter()
            .filter(|ev| ev.ty == "wasm-game_won")
            .collect();
        assert_eq!(
            game_won[0]
                .attributes
                .iter()
                .find(|attr| attr.key == "sank")
                .unwrap()
                .value,
            stringify!((-1, -1))
        );

        let contract_balance: cw20::BalanceResponse = app.wrap()
            .query_wasm_smart(
                cw20_addr.clone(),
                &Cw20QueryMsg::Balance {
                    address: game_addr.to_string(),
            }
        ).unwrap();
        assert_eq!(contract_balance.balance, Uint128::new(100));

        let winner_balance: cw20::BalanceResponse = app.wrap()
            .query_wasm_smart(
                cw20_addr.clone(),
                &Cw20QueryMsg::Balance {
                    address: player2_addr.to_string(),
        },
        )
        .unwrap();

        assert_eq!(winner_balance.balance, Uint128::new(1_000_000 - 1_000 + 1919));

        let token_info: cw20::TokenInfoResponse = app.wrap()
            .query_wasm_smart(
                cw20_addr.clone(),
                &Cw20QueryMsg::TokenInfo {},
            )
        .unwrap();

        assert_eq!(token_info.total_supply, Uint128::new(2_000_000 + 19));

        let err = app
            .execute_contract(
                "player1".into_addr(),
                game_addr.clone(),
                &ExecuteMsg::Play { 
                    field: (1, 0),
                    value: true,
                    proof: vec![]
                },
                &[]
            )
            .unwrap_err();

        assert_eq!(ContractError::GameFinished {  }, err.downcast().unwrap())

    }

    #[test]
    fn should_throw_turn_not_expired() {
        let player1_addr = "player1".into_addr();
        let player2_addr = "player2".into_addr();
        let (_, game_addr, mut app) = init_app(
            player1_addr.clone(),
            player2_addr.clone()
        );

        app
            .execute_contract(
                player1_addr.clone(), 
                game_addr.clone(), 
                &ExecuteMsg::StartGame {}, 
                &[]
        ).unwrap();

        let err = app.execute_contract(
            player2_addr.clone(),
            game_addr.clone(),
            &ExecuteMsg::TimeoutWin {},
            &[]
        ).unwrap_err();

        assert_eq!(ContractError::TurnNotExpired {  }, err.downcast().unwrap())
    }

}
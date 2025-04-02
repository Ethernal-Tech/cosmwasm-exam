#[cfg(test)]
pub mod tests {
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::{App, ContractWrapper, Executor, IntoAddr};
    use crate::{
        contract::{execute, instantiate, query}, 
        msg::{
            AdminResponse, ExecuteMsg, InstantiateMsg, PlayerInstantiate, QueryMsg, ShipsResponse, TurnResponse
        }, state::Player, ContractError
    };

    pub fn mock_instantiate_msg(ships: usize) -> InstantiateMsg {
        InstantiateMsg {
            admin: "admin".into_addr().to_string(),
            ships: ships,
            token_address: "token".into_addr().to_string(),
            players: vec![
                PlayerInstantiate {
                    address: "player1".into_addr().to_string(),
                    stake: Uint128::new(1000),
                    board: vec![
                        vec![false, false, false],
                        vec![false, true, false],
                        vec![false, false, false],
                    ],
                },
                PlayerInstantiate {
                    address: "player2".into_addr().to_string(),
                    stake: Uint128::new(1000),
                    board: vec![
                        vec![false, true, false],
                        vec![false, false, false],
                        vec![false, false, false],
                    ],
                },
            ],
        }
    }

    pub fn init_app() -> (Addr, App) {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id, 
                "owner".into_addr(),
                &mock_instantiate_msg(1), 
                &[], 
                "Contract", 
                None
            ).unwrap();

        (addr, app)
    }

    #[test]
    fn instantiation() {
        let (address,app) = init_app();

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

        let response: TurnResponse = app
            .wrap()
            .query_wasm_smart(address.clone(), &QueryMsg::GetTurn {  })
            .unwrap();

        assert_eq!(response.turn, "player1".into_addr());

        let response: Vec<Player> = app
            .wrap()
            .query_wasm_smart(address.clone(), &QueryMsg::GetPlayers {  })
            .unwrap();

        assert_eq!(response[1].address, "player1".into_addr());
        assert_eq!(response[1].stake, Uint128::new(1000));
        assert_eq!(response[1].board.fields, vec![
            vec![false, false, false],
            vec![false, true, false],
            vec![false, false, false],
        ]);
        assert_eq!(response[1].board.sank, vec![]);

        assert_eq!(response[0].address, "player2".into_addr());
        assert_eq!(response[0].stake, Uint128::new(1000));
        assert_eq!(response[0].board.fields, vec![
            vec![false, true, false],
            vec![false, false, false],
            vec![false, false, false],
        ]);
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
                &mock_instantiate_msg(0), 
                &[], 
                "Contract", 
                None
            ).unwrap_err();

        assert_eq!(ContractError::InvalidShips {  }, err.downcast().unwrap())
    }

    #[test]
    fn should_throw_invalid_board_error() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let err = app
            .instantiate_contract(
                code_id, 
                "owner".into_addr(),
                &mock_instantiate_msg(3), 
                &[], 
                "Contract", 
                None
            ).unwrap_err();

        assert_eq!(ContractError::InvalidBoard {  }, err.downcast().unwrap())
    }

    #[test]
    fn game() {
        let (addr, mut app) = init_app();

        // player1's turn
        let response = app
            .execute_contract(
                "player1".into_addr(),
                addr.clone(),
                &ExecuteMsg::Play { field: (1, 0) },
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
                addr.clone(),
                &ExecuteMsg::Play { field: (1, 1) },
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

        let err = app
            .execute_contract(
                "player1".into_addr(),
                addr.clone(),
                &ExecuteMsg::Play { field: (1, 0) },
                &[]
            )
            .unwrap_err();

        assert_eq!(ContractError::GameFinished {  }, err.downcast().unwrap())

    }

    #[test]
    fn should_throw_wrong_turn_error() {
        let (addr, mut app) = init_app();

        let err = app
            .execute_contract(
                "player2".into_addr(),
                addr.clone(),
                &ExecuteMsg::Play { field: (1, 0) },
                &[]
            )
            .unwrap_err();

        assert_eq!(ContractError::WrongTurn {  }, err.downcast().unwrap())
    }

}
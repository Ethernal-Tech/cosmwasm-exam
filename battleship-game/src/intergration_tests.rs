#[cfg(test)]
pub mod tests {
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::{App, ContractWrapper, Executor, IntoAddr};
    use crate::{
        contract::{execute, instantiate, query}, 
        msg::{
            AdminResponse, InstantiateMsg, PlayerInstantiate, QueryMsg, ShipsResponse, TurnResponse
        }, state::Player, ContractError
    };

    pub fn mock_instantiate_msg(ships: usize) -> InstantiateMsg {
        InstantiateMsg {
            admin: "admin".into_addr().to_string(),
            ships: ships,
            players: vec![
                PlayerInstantiate {
                    address: "player1".into_addr().to_string(),
                    stake: Uint128::new(1000),
                    board: vec![
                        vec![true, false, false],
                        vec![true, true, false],
                        vec![false, false, true],
                    ],
                },
                PlayerInstantiate {
                    address: "player2".into_addr().to_string(),
                    stake: Uint128::new(1000),
                    board: vec![
                        vec![false, true, false],
                        vec![true, false, true],
                        vec![false, true, false],
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
                &mock_instantiate_msg(4), 
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

        assert_eq!(response.ships, 4);

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
            vec![true, false, false],
            vec![true, true, false],
            vec![false, false, true],
        ]);
        assert_eq!(response[1].board.sinked, vec![]);

        assert_eq!(response[0].address, "player2".into_addr());
        assert_eq!(response[0].stake, Uint128::new(1000));
        assert_eq!(response[0].board.fields, vec![
            vec![false, true, false],
            vec![true, false, true],
            vec![false, true, false],
        ]);
        assert_eq!(response[0].board.sinked, vec![]);
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

}
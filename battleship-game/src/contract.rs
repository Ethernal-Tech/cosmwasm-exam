#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult, Uint128
};

use crate::{
    msg::{InstantiateMsg, QueryMsg}, 
    state::{Board, Player, ADMIN, PLAYERS, SHIPS}, ContractError
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg
) -> Result<Response, ContractError> {
    let admin = deps.api.addr_validate(&msg.admin)?;
    ADMIN.save(deps.storage, &admin)?;

    let ships = msg.ships;
    if ships == 0 {
        return Err(ContractError::InvalidShips {});
    }
    SHIPS.save(deps.storage, &Uint128::new(ships))?;

    for player in msg.players {
        let address = deps.api.addr_validate(&player.address)?;
        let stake = player.stake;
        
        if !validate_board(&player.board, ships) {
            return Err(ContractError::InvalidBoard {});
        }

        let board = Board {
            fields: player.board,
            sinked: vec![],
        };

        let player = Player {
            address: address.clone(),
            stake: stake,
            board: board,
        };

        PLAYERS.save(deps.storage, address, &player)?;
    }

    Ok(Response::new())
}

pub fn validate_board(board: &Vec<Vec<bool>>, ships: u128) -> bool {
    let mut count = 0u128;

    for row in board {
        for cell in row {
            if *cell {
                count += 1;
            }
        }
    }

    count == ships
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty
) -> Response {
    unimplemented!();
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAdmin {} => to_json_binary(&query::get_admin(deps)?),
        QueryMsg::GetPlayers {} => to_json_binary(&query::get_players(deps)?),
        QueryMsg::GetShips {} => to_json_binary(&query::get_ships(deps)?),
        QueryMsg::GetTurn {} => to_json_binary(&query::get_turn(deps)?)
    }
}

mod execute {
    
}

mod query {
    use cosmwasm_std::{Addr, Order};

    use crate::state::TURN;

    use super::*;

    pub fn get_admin(deps: Deps) -> StdResult<Addr> {
        let admin = ADMIN.load(deps.storage);
        Ok(admin?)
    }

    pub fn get_players(deps: Deps) -> StdResult<Vec<Player>> {
        PLAYERS
            .range(deps.storage, None, None, Order::Ascending)
            .map(|item| {
                let (_addr, player) = item?;
                Ok(player)
            })
            .collect()
    }

    pub fn get_ships(deps: Deps) -> StdResult<Uint128> {
        let ships = SHIPS.load(deps.storage);
        Ok(ships?)
    }

    pub fn get_turn(deps: Deps) -> StdResult<Addr> {
        let turn = TURN.load(deps.storage);
        Ok(turn?)
    }
}

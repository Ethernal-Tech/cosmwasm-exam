#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult
};

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg}, 
    state::{Board, Player, ADMIN, FINISHED, PLAYERS, SHIPS, TURN}, ContractError
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
    SHIPS.save(deps.storage, &ships)?;

    TURN.save(deps.storage, &deps.api.addr_validate(&msg.players[0].address)?)?;

    FINISHED.save(deps.storage, &false)?;

    for player in msg.players {
        let address = deps.api.addr_validate(&player.address)?;
        let stake = player.stake;
        
        if !validate_board(&player.board, ships) {
            return Err(ContractError::InvalidBoard {});
        }

        let board = Board {
            fields: player.board,
            sank: vec![],
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

pub fn validate_board(board: &Vec<Vec<bool>>, ships: usize) -> bool {
    let mut count = 0;

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
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Play {field} => execute::play(deps, info, field)
    }
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
    use cosmwasm_std::{Event, Order};

    use super::*;

    pub fn play(
        deps: DepsMut,
        info: MessageInfo,
        field: (usize, usize)
    ) -> Result<Response, ContractError> {
        if FINISHED.load(deps.storage)? {
            return Err(ContractError::GameFinished {});
        }

        let player = info.sender;
        if player != TURN.load(deps.storage)? {
            return Err(ContractError::WrongTurn {  })
        }

        let oponent = PLAYERS
            .range(deps.storage, None, None, Order::Ascending)
            .find_map(|item| {
                let (addr, player_data) = item.ok()?;
                if addr != player {
                    Some(player_data)
                } else {
                    None
                }
        })
        .ok_or(ContractError::PlayerNotFound {  });

        let oponent = oponent?;

        let oponent_sinked = &oponent.board.sank;
        if oponent_sinked.contains(&field) {
            return Err(ContractError::AlreadySunk {});
        }

        let oponent_board = &oponent.board.fields;
        if oponent_board[field.0][field.1] {
            let oponent = PLAYERS.update::<_, ContractError>(deps.storage, oponent.address.clone(), |player| {
                let mut player = player.ok_or(ContractError::PlayerNotFound {})?;
                player.board.sank.push(field);
                Ok(player)
            })?;

            let mut event = "ship_sank";
            if oponent.board.sank.len() == SHIPS.load(deps.storage)? {
                event = "game_won";
                FINISHED.update::<_, ContractError>(deps.storage, |_| Ok(true))?;
            }

            TURN.update::<_, ContractError>(deps.storage, |_| Ok(oponent.address.clone()))?;
            return Ok(
                Response::new()
                    .add_attribute("action", "play")
                    .add_event(Event::new(event).add_attribute("sank", format!("{:?}", field)))
            );
        }

        TURN.update::<_, ContractError>(deps.storage, |_| Ok(oponent.address.clone()))?;
        Ok(
            Response::new()
                .add_attribute("action", "play")
                .add_event(Event::new("ship_missed").add_attribute("missed", format!("{:?}", field)))
        )

    }
}

mod query {
    use cosmwasm_std::Order;

    use crate::{msg::{AdminResponse, ShipsResponse, TurnResponse}, state::TURN};

    use super::*;

    pub fn get_admin(deps: Deps) -> StdResult<AdminResponse> {
        let admin = ADMIN.load(deps.storage);
        Ok(AdminResponse { admin: admin? })
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

    pub fn get_ships(deps: Deps) -> StdResult<ShipsResponse> {
        let ships = SHIPS.load(deps.storage);
        Ok(ShipsResponse { ships: ships? })
    }

    pub fn get_turn(deps: Deps) -> StdResult<TurnResponse> {
        let turn = TURN.load(deps.storage);
        Ok(TurnResponse { turn: turn? })
    }
}

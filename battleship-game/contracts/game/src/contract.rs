#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128
};

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg}, 
    state::{Board, Player, ADMIN, FINISHED, MIN_STAKE, PLAYERS, SHIPS, STARTED, TOKEN_ADDRESS, TURN}, ContractError
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg
) -> Result<Response, ContractError> {
    ADMIN.save(deps.storage, &deps.api.addr_validate(&msg.admin)?)?;

    TOKEN_ADDRESS.save(deps.storage, &deps.api.addr_validate(&msg.token_address)?)?;

    STARTED.save(deps.storage, &false)?;

    let ships = msg.ships;
    if ships == 0 {
        return Err(ContractError::InvalidShips {});
    }
    SHIPS.save(deps.storage, &ships)?;

    TURN.save(deps.storage, &deps.api.addr_validate(&msg.players[0].address)?)?;

    FINISHED.save(deps.storage, &false)?;

    if msg.players[0].stake < Uint128::new(MIN_STAKE) {
        return Err(ContractError::InvalidStake {})
    }

    if msg.players[0].stake != msg.players[1].stake {
        return Err(ContractError::InvalidStake {})
    }

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
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::StartGame {} => execute::start_game(deps, env, info),
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
    use std::vec;

    use cosmwasm_std::{Addr, Event, Order};
    use cw20::Cw20ExecuteMsg;

    use crate::state::{FEE_PERCENTAGE, REWARD_PERCENTAGE};

    use super::*;

    pub fn start_game(
        deps:DepsMut,
        env: Env,
        info: MessageInfo
    ) -> Result<Response, ContractError> {
        if FINISHED.load(deps.storage)? {
            return Err(ContractError::GameFinished {});
        }

        if STARTED.load(deps.storage)? {
            return Err(ContractError::GameStarted {});
        }

        let caller = info.sender.clone();
        let players: Vec<Player> = PLAYERS
            .range(deps.storage, None, None, Order::Ascending)
            .map(|res| res.map(|(_, player)| player))
            .collect::<StdResult<Vec<Player>>>()?;
        if players.iter().all(|player| player.address != caller) {
            return Err(ContractError::Unauthorized{});
        }

        let token_addr = TOKEN_ADDRESS.load(deps.storage)?;
        let mut messages: Vec<cosmwasm_std::CosmosMsg> = vec![];

        for player in &players {
            let transfer_msg = Cw20ExecuteMsg::TransferFrom {
                owner: player.address.to_string(),
                recipient: env.contract.address.to_string(),
                amount: player.stake
            };
            // potencijalni problem zbog to_json_binary
            messages.push(
                cosmwasm_std::WasmMsg::Execute {
                    contract_addr: token_addr.to_string(),
                    msg: to_json_binary(&transfer_msg)?, 
                    funds: vec![]
                }.into()
            );
        }

        STARTED.save(deps.storage, &true)?;

        Ok(Response::new()
            .add_attribute("action", "start_game")
            .add_attribute("stake", players[0].stake.to_string())
            .add_message(messages[0].clone())
            .add_message(messages[1].clone())
        )
    }

    pub fn play(
        deps: DepsMut,
        info: MessageInfo,
        field: (usize, usize)
    ) -> Result<Response, ContractError> {
        if !STARTED.load(deps.storage)? {
            return Err(ContractError::GameNotStarted {});
        }

        if FINISHED.load(deps.storage)? {
            return Err(ContractError::GameFinished {});
        }

        let player = info.sender;
        if player != TURN.load(deps.storage)? {
            return Err(ContractError::WrongTurn {  })
        }
        let player = PLAYERS.load(deps.storage, player)?;

        let oponent = PLAYERS
            .range(deps.storage, None, None, Order::Ascending)
            .find_map(|item| {
                let (addr, player_data) = item.ok()?;
                if addr != player.address {
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

            if oponent.board.sank.len() == SHIPS.load(deps.storage)? {
                FINISHED.update::<_, ContractError>(deps.storage, |_| Ok(true))?;

                let total_amount = player.stake + oponent.stake;
                let fee = total_amount.multiply_ratio(FEE_PERCENTAGE, 100u128);
                let payout = total_amount.checked_sub(fee)
                    .map_err(|_| ContractError::Overflow {})?;
                let token_address = TOKEN_ADDRESS.load(deps.storage)?;

                //transfer funds to winner
                let transfer_msg = transfer(
                    player.address.clone(), 
                    payout, 
                    token_address.clone()
                )?;

                // mint reword for winner
                let reward = payout.multiply_ratio(REWARD_PERCENTAGE, 100u128);
                let mint_msg = mint(
                    player.address.clone(), 
                    reward, 
                    token_address
                )?;

                return Ok(Response::new()
                    .add_attribute("action", "play")
                    .add_attribute("winner", player.address.to_string())
                    .add_attribute("payout", payout.to_string())
                    .add_attribute("fee_retained", fee.to_string())
                    .add_event(Event::new("game_won").add_attribute("sank", format!("{:?}", field)))
                    .add_message(transfer_msg)
                    .add_attribute("minted_reward", reward.to_string())
                    .add_message(mint_msg)
                );
            }

            TURN.update::<_, ContractError>(deps.storage, |_| Ok(oponent.address.clone()))?;
            return Ok(
                Response::new()
                    .add_attribute("action", "play")
                    .add_event(Event::new("ship_sank").add_attribute("sank", format!("{:?}", field)))
            );
        }

        TURN.update::<_, ContractError>(deps.storage, |_| Ok(oponent.address.clone()))?;
        Ok(
            Response::new()
                .add_attribute("action", "play")
                .add_event(Event::new("ship_missed").add_attribute("missed", format!("{:?}", field)))
        )

    }

    pub fn transfer(
        recipient_addr: Addr,
        amount: Uint128,
        token_addr: Addr
    ) -> Result<cosmwasm_std::WasmMsg, cosmwasm_std::StdError> {
        let transfer_msg = Cw20ExecuteMsg::Transfer { 
            recipient: recipient_addr.to_string(), 
            amount: amount
        };
        Ok(cosmwasm_std::WasmMsg::Execute {
            contract_addr: token_addr.to_string(),
            msg: to_json_binary(&transfer_msg)?,
            funds: vec![],
        })
    }

    pub fn mint(
        recipient_addr: Addr,
        amount: Uint128,
        token_address: Addr
    ) -> Result<cosmwasm_std::WasmMsg, cosmwasm_std::StdError> {
        let mint_msg = Cw20ExecuteMsg::Mint {
            recipient: recipient_addr.to_string(),
            amount: amount
        };
        Ok(cosmwasm_std::WasmMsg::Execute {
            contract_addr: token_address.to_string(), 
            msg: to_json_binary(&mint_msg)?,
            funds: vec![] 
        })
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

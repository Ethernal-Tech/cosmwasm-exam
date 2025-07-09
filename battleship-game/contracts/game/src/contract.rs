#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128
};

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg}, 
    state::{Board, GameConfig, GameState, Player, GAME_CONFIG, GAME_STATE, MIN_STAKE, PLAYERS}, ContractError
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg
) -> Result<Response, ContractError> {

    let ships = msg.ships;
    if ships == 0 {
        return Err(ContractError::InvalidShips {});
    }
    let game_config = GameConfig { 
        token_address: deps.api.addr_validate(&msg.token_address)?, 
        ships: ships 
    };
    GAME_CONFIG.save(deps.storage, &game_config)?;

    let game_state = GameState { 
        started: false, 
        finished: false, 
        turn: deps.api.addr_validate(&msg.players[0].address)?, 
        last_turn_time: 0 
    };
    GAME_STATE.save(deps.storage, &game_state)?;

    if msg.players[0].stake < Uint128::new(MIN_STAKE) {
        return Err(ContractError::InvalidStake {})
    }

    if msg.players[0].stake != msg.players[1].stake {
        return Err(ContractError::InvalidStake {})
    }

    for player in msg.players {
        let address = deps.api.addr_validate(&player.address)?;
        let stake = player.stake;

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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::StartGame {} => 
            execute::start_game(deps, env, info),
        ExecuteMsg::Play {field, value, proof} => 
            execute::play(deps, env, info, field, value, proof),
        ExecuteMsg::TimeoutWin {} => 
            execute::timeout_win(deps, env, info)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPlayers {} => to_json_binary(&query::get_players(deps)?),
        QueryMsg::GetGameConfig {} => to_json_binary(&query::get_game_config(deps)?),
        QueryMsg::GetGameState {} => to_json_binary(&query::get_game_state(deps)?),
    }
}

mod execute {
    use cosmwasm_std::{Addr, Event, Order};
    use cw20::Cw20ExecuteMsg;
    use sha2::{Digest, Sha256};
    use hex;

    use crate::{msg::ProofStep, state::{FEE_PERCENTAGE, REWARD_PERCENTAGE, TURN_DURATION}};

    use super::*;

    pub fn start_game(
        deps:DepsMut,
        env: Env,
        info: MessageInfo
    ) -> Result<Response, ContractError> {
        let game_config = GAME_CONFIG.load(deps.storage)?;
        let mut game_state = GAME_STATE.load(deps.storage)?;

        if game_state.started {
            return Err(ContractError::GameStarted {});
        }

        if game_state.finished {
            return Err(ContractError::GameFinished {});
        }

        let caller = info.sender.clone();
        let players: Vec<Player> = PLAYERS
            .range(deps.storage, None, None, Order::Ascending)
            .map(|res| res.map(|(_, player)| player))
            .collect::<StdResult<Vec<Player>>>()?;
        if players.iter().all(|player| player.address != caller) {
            return Err(ContractError::Unauthorized{});
        }

        let token_addr = game_config.token_address;
        let mut messages: Vec<cosmwasm_std::CosmosMsg> = vec![];

        for player in &players {
            let transfer_msg = Cw20ExecuteMsg::TransferFrom {
                owner: player.address.to_string(),
                recipient: env.contract.address.to_string(),
                amount: player.stake
            };
            messages.push(
                cosmwasm_std::WasmMsg::Execute {
                    contract_addr: token_addr.to_string(),
                    msg: to_json_binary(&transfer_msg)?, 
                    funds: vec![]
                }.into()
            );
        }

        game_state.started = true;
        game_state.last_turn_time = env.block.time.seconds();
        GAME_STATE.save(deps.storage, &game_state)?;

        Ok(Response::new()
            .add_attribute("action", "start_game")
            .add_attribute("stake", players[0].stake.to_string())
            .add_message(messages[0].clone())
            .add_message(messages[1].clone())
        )
    }

    pub fn play(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        field: (usize, usize),
        field_value: bool,
        proof: Vec<ProofStep>
    ) -> Result<Response, ContractError> {
        let game_config = GAME_CONFIG.load(deps.storage)?;
        let mut game_state = GAME_STATE.load(deps.storage)?;

        if !game_state.started {
            return Err(ContractError::GameNotStarted {});
        }

        if game_state.finished {
            return Err(ContractError::GameFinished {});
        }

        if env.block.time.seconds() > game_state.last_turn_time + TURN_DURATION {
            return Err(ContractError::TurnExpired {  });
        }

        let player = info.sender;
        if player != game_state.turn {
            return Err(ContractError::WrongTurn {  })
        }
        let player = PLAYERS.load(deps.storage, player)?;

        let opponent = PLAYERS
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

        let opponent = opponent?;

        if !verify_proof(field_value, proof, &opponent.board.fields) {
            return Err(ContractError::InvalidProof {  });
        }

        let opponent_sunk = &opponent.board.sank;
        if opponent_sunk.contains(&field) {
            return Err(ContractError::AlreadySunk {});
        }

        game_state.last_turn_time = env.block.time.seconds();
        if field_value {
            let opponent = PLAYERS
                .update::<_, ContractError>(
                    deps.storage, 
                    opponent.address.clone(), 
                    |player| {
                        let mut player = player.ok_or(ContractError::PlayerNotFound {})?;
                        player.board.sank.push(field);
                        Ok(player)
                    }
                )?;

            if opponent.board.sank.len() == game_config.ships {
                game_state.finished = true;

                let total_amount = player.stake + opponent.stake;
                let fee = total_amount.multiply_ratio(FEE_PERCENTAGE, 100u128);
                let payout = total_amount.checked_sub(fee)
                    .map_err(|_| ContractError::Overflow {})?;
                let token_address = game_config.token_address;

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

                GAME_STATE.save(deps.storage, &game_state)?;

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

            game_state.turn = opponent.address;
            GAME_STATE.save(deps.storage, &game_state)?;
            return Ok(
                Response::new()
                    .add_attribute("action", "play")
                    .add_event(Event::new("ship_sank").add_attribute("sank", format!("{:?}", field)))
            );
        }

        game_state.turn = opponent.address;
        GAME_STATE.save(deps.storage, &game_state)?;
        Ok(
            Response::new()
                .add_attribute("action", "play")
                .add_event(Event::new("ship_missed").add_attribute("missed", format!("{:?}", field)))
        )

    }

    pub fn verify_proof(value: bool, proof: Vec<ProofStep>, merkle_root: &str) -> bool {
        let mut current_hash = hash(value.to_string());
        println!("{}", value);

        for step in proof {
            println!("{}", current_hash);
            if step.is_left {
                current_hash = hash(step.hash + &current_hash);
                continue;
            }
            current_hash = hash(current_hash + &step.hash);
        }

        return current_hash == merkle_root;
    }

    pub fn hash(item: String) -> String {
        let mut hasher = Sha256::new();
        hasher.update(item.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    pub fn timeout_win(
        deps: DepsMut,
        env: Env,
        info: MessageInfo
    ) -> Result<Response, ContractError> {
        let game_config = GAME_CONFIG.load(deps.storage)?;
        let mut game_state = GAME_STATE.load(deps.storage)?;

        if !game_state.started {
            return Err(ContractError::GameNotStarted {});
        }

        if game_state.finished {
            return Err(ContractError::GameFinished {});
        }

        let player = PLAYERS.load(deps.storage, info.sender)?;
        let opponent_address = game_state.turn.clone();

        if player.address == opponent_address {
            return Err(ContractError::Unauthorized {  })
        }

        let now = env.block.time.seconds();
        if now <= game_state.last_turn_time + TURN_DURATION {
            return Err(ContractError::TurnNotExpired {  });
        }

        game_state.finished = true;

        let opponent = PLAYERS.load(deps.storage, opponent_address)?;
        let total_amount = player.stake + opponent.stake;
        let fee = total_amount.multiply_ratio(FEE_PERCENTAGE, 100u128);
        let payout = total_amount.checked_sub(fee)
            .map_err(|_| ContractError::Overflow {})?;
        let token_address = game_config.token_address;

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

        GAME_STATE.save(deps.storage, &game_state)?;

        return Ok(Response::new()
            .add_attribute("action", "timeout_check")
            .add_attribute("winner", player.address.to_string())
            .add_attribute("payout", payout.to_string())
            .add_attribute("fee_retained", fee.to_string())
            .add_event(Event::new("game_won").add_attribute("sank", format!("{:?}", (-1, -1))))
            .add_message(transfer_msg)
            .add_attribute("minted_reward", reward.to_string())
            .add_message(mint_msg)
        );
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

    use super::*;

    pub fn get_players(deps: Deps) -> StdResult<Vec<Player>> {
        PLAYERS
            .range(deps.storage, None, None, Order::Ascending)
            .map(|item| {
                let (_addr, player) = item?;
                Ok(player)
            })
            .collect()
    }

    pub fn get_game_config(deps: Deps) -> StdResult<GameConfig> {
        let game_config = GAME_CONFIG.load(deps.storage)?;
        Ok(game_config)
    }

    pub fn get_game_state(deps: Deps) -> StdResult<GameState> {
        let game_state = GAME_STATE.load(deps.storage)?;
        Ok(game_state)
    }

}

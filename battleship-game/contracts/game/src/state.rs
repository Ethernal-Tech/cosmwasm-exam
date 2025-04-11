use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct GameConfig {
    pub token_address: Addr,
    pub ships: usize,
}

pub const GAME_CONFIG: Item<GameConfig> = Item::new("game_config");

#[cw_serde]
pub struct GameState {
    pub started: bool,
    pub finished: bool,
    pub turn: Addr,
    pub last_turn_time: u64,
}

pub const GAME_STATE: Item<GameState> = Item::new("game_state");

#[cw_serde]
pub struct Board {
    pub fields: String,
    pub sank: Vec<(usize, usize)>,
}

#[cw_serde]
pub struct Player {
    pub address: Addr,
    pub stake: Uint128,
    pub board: Board,
}

// game boards (map): addr: player, each player has his own staked assets and a board
pub const PLAYERS: Map<Addr, Player> = Map::new("players");

// constants for rewards
pub const MIN_STAKE: u128 = 50u128;
pub const REWARD_PERCENTAGE: u128 = 1u128;
pub const FEE_PERCENTAGE: u128 = 5u128;
pub const TURN_DURATION: u64 = 60u64;

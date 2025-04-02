use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

// admin account, for minting
pub const ADMIN: Item<Addr> = Item::new("admin");

// token contract address
pub const TOKEN_ADDRESS: Item<Addr> = Item::new("token_address");

// flag for game start
pub const STARTED: Item<bool> = Item::new("started");

// amount of ships to sink
pub const SHIPS: Item<usize> = Item::new("ships");

#[cw_serde]
pub struct Board {
    pub fields: Vec<Vec<bool>>,
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

// current turn
pub const TURN: Item<Addr> = Item::new("turn");

// game finished
pub const FINISHED: Item<bool> = Item::new("finished");

pub const MIN_STAKE: u128 = 50u128;
pub const REWARD_PERCENTAGE: u128 = 10u128;
pub const FEE_PERCENTAGE: u128 = 50u128;

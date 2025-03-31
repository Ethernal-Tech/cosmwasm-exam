use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

// admin account, for minting
pub const ADMIN: Item<Addr> = Item::new("admin");

pub struct Board {
    pub fields: Vec<Vec<bool>>,
    pub sinked: Vec<(u64, u64)>,
}

pub struct Player {
    pub address: Addr,
    pub stake: Uint128,
    pub board: Board,
}

// game boards (map): addr: player, each player has his own staked assets and a board
pub const PLAYERS: Map<Addr, Player> = Map::new("players");

// current turn
pub const TURN: Item<Addr> = Item::new("turn");

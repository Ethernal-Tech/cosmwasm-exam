use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

use crate::state::Player;

#[cw_serde]
pub struct PlayerInstantiate {
    pub address: String,
    pub stake: Uint128,
    pub board: Vec<Vec<bool>>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub ships: u128,
    pub players: Vec<PlayerInstantiate>,
}

#[cw_serde]
pub enum QueryMsg {
    GetAdmin {},
    GetPlayers {},
    GetTurn {},
    GetShips {},
}

#[cw_serde]
pub enum ExecuteMsg {
    Play {},
}

#[cw_serde]
pub struct AdminResponse {
    pub admin: Addr
}

#[cw_serde]
pub struct PlayersResponse {
    pub players: Vec<Player>
}

#[cw_serde]
pub struct ShipsResponse {
    pub ships: Uint128
}

#[cw_serde]
pub struct TurnResponse {
    pub turn: Addr
}
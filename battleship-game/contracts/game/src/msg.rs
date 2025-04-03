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
    pub token_address: String,
    pub ships: usize,
    pub players: Vec<PlayerInstantiate>,
}

#[cw_serde]
pub enum QueryMsg {
    GetAdmin {},
    GetPlayers {},
    GetTurn {},
    GetShips {},
    GetStarted {},
    GetFinished {},
    GetTokenAddress {},
}

#[cw_serde]
pub enum ExecuteMsg {
    StartGame {},
    Play {field: (usize, usize)},
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
    pub ships: usize
}

#[cw_serde]
pub struct AddressResponse {
    pub address: Addr
}

#[cw_serde]
pub struct BoolResponse {
    pub value: bool
}



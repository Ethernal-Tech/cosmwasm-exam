use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

use crate::state::Player;

#[cw_serde]
pub struct PlayerInstantiate {
    pub address: String,
    pub stake: Uint128,
    pub board: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub token_address: String,
    pub ships: usize,
    pub players: Vec<PlayerInstantiate>,
}

#[cw_serde]
pub enum QueryMsg {
    GetPlayers {},
    GetGameConfig {},
    GetGameState {}
}

#[cw_serde]
pub struct ProofStep {
    pub hash: String,
    pub is_left: bool,
}

#[cw_serde]
pub enum ExecuteMsg {
    StartGame {},
    Play {
        field: (usize, usize),
        value: bool,
        proof: Vec<ProofStep>
    },
    TimeoutWin {},
}

#[cw_serde]
pub struct PlayersResponse {
    pub players: Vec<Player>
}


